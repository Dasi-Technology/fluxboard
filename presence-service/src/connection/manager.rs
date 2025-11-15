use crate::connection::room::Room;
use crate::connection::session::Session;
use crate::protocol::messages::BinaryMessage;
use crate::redis::pubsub::{RedisMessage, RedisPubSub};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Manages all WebSocket connections, sessions, and rooms
#[derive(Clone)]
pub struct ConnectionManager {
    /// Map of client addresses to their message senders
    connections: Arc<RwLock<HashMap<SocketAddr, UnboundedSender<Message>>>>,

    /// Map of client addresses to their session info
    sessions: Arc<RwLock<HashMap<SocketAddr, Session>>>,

    /// Map of board IDs to rooms
    rooms: Arc<RwLock<HashMap<u16, Room>>>,

    /// Redis pub/sub for multi-instance coordination
    redis_pubsub: Arc<RedisPubSub>,

    /// Unique identifier for this service instance
    instance_id: String,
}

impl ConnectionManager {
    /// Create a new ConnectionManager with Redis pub/sub support
    pub fn new(redis_pubsub: Arc<RedisPubSub>) -> Self {
        let instance_id = Uuid::new_v4().to_string();
        info!(
            "Creating ConnectionManager with instance ID: {}",
            instance_id
        );

        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            redis_pubsub,
            instance_id,
        }
    }

    /// Start listening for Redis pub/sub messages
    pub async fn start_redis_listener(self: Arc<Self>) {
        info!(
            "Starting Redis pub/sub listener for instance {}",
            self.instance_id
        );

        // We'll subscribe to channels dynamically as boards are joined
        // For now, subscribe to the global channel
        let channels = vec![RedisPubSub::global_channel()];

        tokio::spawn(async move {
            self.subscribe_with_retry(channels).await;
        });
    }

    /// Subscribe to Redis channels with automatic retry
    async fn subscribe_with_retry(&self, channels: Vec<String>) {
        loop {
            match self.redis_pubsub.subscribe(channels.clone()).await {
                Ok(mut stream) => {
                    info!("Successfully subscribed to Redis channels");

                    // Process incoming messages
                    while let Some((channel, redis_msg)) = stream.recv().await {
                        // Skip messages from this instance (avoid echo)
                        if redis_msg.instance_id == self.instance_id {
                            debug!("Skipping message from own instance");
                            continue;
                        }

                        // Decode the binary message
                        match redis_msg.get_binary_message() {
                            Ok(binary_msg) => {
                                self.handle_redis_message(&channel, binary_msg).await;
                            }
                            Err(e) => {
                                error!("Failed to decode binary message from Redis: {}", e);
                            }
                        }
                    }

                    warn!("Redis subscription stream ended, reconnecting...");
                }
                Err(e) => {
                    error!("Failed to subscribe to Redis: {}, retrying in 5s...", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }

            // Add a small delay before retry to avoid tight loop
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Handle incoming messages from Redis
    async fn handle_redis_message(&self, channel: &str, message: BinaryMessage) {
        debug!(
            "Received Redis message on channel {}: {:?}",
            channel, message
        );

        match &message {
            BinaryMessage::UserJoined { board_id, .. }
            | BinaryMessage::UserLeft { board_id, .. }
            | BinaryMessage::CursorBroadcast { board_id, .. }
            | BinaryMessage::PresenceUpdate { board_id, .. } => {
                // Broadcast to local WebSocket clients in this room
                self.broadcast_to_room(*board_id, message, None).await;
            }
            _ => {
                debug!("Ignoring non-broadcast message from Redis: {:?}", message);
            }
        }
    }

    /// Publish a message to Redis
    async fn publish_to_redis(&self, board_id: u16, message: &BinaryMessage) {
        let channel = RedisPubSub::board_channel(board_id);
        let redis_msg = RedisMessage::new(self.instance_id.clone(), message);

        match redis_msg.encode() {
            Ok(encoded) => {
                if let Err(e) = self.redis_pubsub.publish(&channel, &encoded).await {
                    warn!("Failed to publish to Redis channel {}: {}", channel, e);
                    // Continue anyway - local broadcasting will still work
                }
            }
            Err(e) => {
                error!("Failed to encode Redis message: {}", e);
            }
        }
    }

    /// Register a new connection
    pub async fn connect(&self, addr: SocketAddr, tx: UnboundedSender<Message>) {
        let mut connections = self.connections.write().await;
        connections.insert(addr, tx);

        let mut sessions = self.sessions.write().await;
        sessions.insert(addr, Session::new(addr));

        info!("Client connected: {}", addr);
    }

    /// Handle client disconnect
    pub async fn disconnect(&self, addr: SocketAddr) {
        info!("Client disconnecting: {}", addr);

        // Get the session to find which rooms the client is in
        let session = {
            let sessions = self.sessions.read().await;
            sessions.get(&addr).cloned()
        };

        if let Some(session) = session {
            // Remove client from all rooms they're in
            let board_ids: Vec<u16> = session.board_ids().iter().copied().collect();
            for board_id in board_ids {
                self.handle_leave_internal(addr, board_id).await;
            }
        }

        // Remove connection and session
        let mut connections = self.connections.write().await;
        connections.remove(&addr);

        let mut sessions = self.sessions.write().await;
        sessions.remove(&addr);

        debug!("Client disconnected and cleaned up: {}", addr);
    }

    /// Handle incoming messages from clients
    pub async fn handle_message(&self, addr: SocketAddr, msg: BinaryMessage) {
        match msg {
            BinaryMessage::Join { board_id, username } => {
                self.handle_join(addr, board_id, username).await;
            }
            BinaryMessage::Leave { board_id } => {
                self.handle_leave(addr, board_id).await;
            }
            BinaryMessage::CursorUpdate { board_id, x, y } => {
                self.handle_cursor_update(addr, board_id, x, y).await;
            }
            BinaryMessage::Heartbeat => {
                self.handle_heartbeat(addr).await;
            }
            _ => {
                warn!("Received unexpected server message from client: {:?}", msg);
            }
        }
    }

    /// Handle Join message
    async fn handle_join(&self, addr: SocketAddr, board_id: u16, username: String) {
        debug!("Client {} joining board {}", addr, board_id);

        // Check if client is already in the room
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(&addr) {
                if session.board_ids().contains(&board_id) {
                    warn!("Client {} already in room {}", addr, board_id);
                    return;
                }
            }
        }

        // Get or create room and assign user ID
        let (user_id, color, user_count) = {
            let mut rooms = self.rooms.write().await;
            let room = rooms.entry(board_id).or_insert_with(|| Room::new(board_id));

            // Assign user ID
            let user_id = match room.assign_user_id() {
                Some(id) => id,
                None => {
                    error!("Room {} is full (max 255 users)", board_id);
                    return;
                }
            };

            // Generate random color for cursor
            let color = Self::generate_color();

            // Add user to room
            room.add_user(addr, user_id, username.clone(), color);

            let user_count = room.user_count();

            (user_id, color, user_count)
        };

        // Update session
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&addr) {
                session.add_board(board_id, user_id, username.clone(), color);
            }
        }

        info!(
            "Client {} joined board {} as user {} ({})",
            addr, board_id, user_id, username
        );

        // Broadcast UserJoined to other room members (local and remote)
        let user_joined = BinaryMessage::UserJoined {
            board_id,
            user_id,
            username: username.clone(),
            color,
        };

        // Publish to Redis for other instances
        self.publish_to_redis(board_id, &user_joined).await;

        // Broadcast locally
        self.broadcast_to_room(board_id, user_joined, Some(addr))
            .await;

        // Send PresenceUpdate to all room members (including the new user)
        let presence_update = BinaryMessage::PresenceUpdate {
            board_id,
            count: user_count as u8,
        };

        // Publish to Redis for other instances
        self.publish_to_redis(board_id, &presence_update).await;

        // Broadcast locally
        self.broadcast_to_room(board_id, presence_update, None)
            .await;
    }

    /// Handle Leave message
    async fn handle_leave(&self, addr: SocketAddr, board_id: u16) {
        self.handle_leave_internal(addr, board_id).await;
    }

    /// Internal leave handler (used by both explicit leave and disconnect)
    async fn handle_leave_internal(&self, addr: SocketAddr, board_id: u16) {
        debug!("Client {} leaving board {}", addr, board_id);

        // Get user info before removing
        let user_id = {
            let sessions = self.sessions.read().await;
            match sessions.get(&addr).and_then(|s| s.get_board_info(board_id)) {
                Some(info) => info.user_id,
                None => {
                    warn!("Client {} not in room {}", addr, board_id);
                    return;
                }
            }
        };

        // Remove user from room and check if room should be deleted
        let (should_delete_room, user_count) = {
            let mut rooms = self.rooms.write().await;
            if let Some(room) = rooms.get_mut(&board_id) {
                room.remove_user(addr);
                let count = room.user_count();
                (count == 0, count)
            } else {
                warn!("Room {} does not exist", board_id);
                return;
            }
        };

        // Update session
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&addr) {
                session.remove_board(board_id);
            }
        }

        info!("Client {} left board {} (user {})", addr, board_id, user_id);

        // Broadcast UserLeft to remaining room members (local and remote)
        let user_left = BinaryMessage::UserLeft { board_id, user_id };

        // Publish to Redis for other instances
        self.publish_to_redis(board_id, &user_left).await;

        // Broadcast locally
        self.broadcast_to_room(board_id, user_left, Some(addr))
            .await;

        // Send PresenceUpdate to remaining room members
        if user_count > 0 {
            let presence_update = BinaryMessage::PresenceUpdate {
                board_id,
                count: user_count as u8,
            };

            // Publish to Redis for other instances
            self.publish_to_redis(board_id, &presence_update).await;

            // Broadcast locally
            self.broadcast_to_room(board_id, presence_update, None)
                .await;
        }

        // Clean up empty room
        if should_delete_room {
            let mut rooms = self.rooms.write().await;
            rooms.remove(&board_id);
            debug!("Removed empty room {}", board_id);
        }
    }

    /// Handle CursorUpdate message
    async fn handle_cursor_update(&self, addr: SocketAddr, board_id: u16, x: u16, y: u16) {
        // Get user ID from session
        let user_id = {
            let sessions = self.sessions.read().await;
            match sessions.get(&addr) {
                Some(session) => match session.get_board_info(board_id) {
                    Some(info) => info.user_id,
                    None => {
                        warn!("Client {} not in room {}", addr, board_id);
                        return;
                    }
                },
                None => {
                    warn!("Session not found for {}", addr);
                    return;
                }
            }
        };

        // Broadcast cursor position to other room members (local and remote)
        let cursor_broadcast = BinaryMessage::CursorBroadcast {
            board_id,
            user_id,
            x,
            y,
        };

        // Publish to Redis for other instances
        self.publish_to_redis(board_id, &cursor_broadcast).await;

        // Broadcast locally
        self.broadcast_to_room(board_id, cursor_broadcast, Some(addr))
            .await;
    }

    /// Handle Heartbeat message
    async fn handle_heartbeat(&self, addr: SocketAddr) {
        debug!("Heartbeat from {}", addr);

        // Send heartbeat response
        let heartbeat = BinaryMessage::Heartbeat;
        if let Err(e) = self.send_to_client(addr, heartbeat).await {
            warn!("Failed to send heartbeat to {}: {}", addr, e);
        }
    }

    /// Broadcast a message to all users in a room
    async fn broadcast_to_room(
        &self,
        board_id: u16,
        message: BinaryMessage,
        exclude: Option<SocketAddr>,
    ) {
        // Get all user addresses in the room
        let user_addrs = {
            let rooms = self.rooms.read().await;
            match rooms.get(&board_id) {
                Some(room) => room.user_addresses().iter().copied().collect::<Vec<_>>(),
                None => {
                    debug!("Room {} does not exist for broadcast", board_id);
                    return;
                }
            }
        };

        // Encode message once
        let encoded = message.encode();
        let ws_message = Message::Binary(encoded.into());

        // Send to all users except the excluded one
        let connections = self.connections.read().await;
        for user_addr in user_addrs {
            if let Some(excluded) = exclude {
                if user_addr == excluded {
                    continue;
                }
            }

            if let Some(tx) = connections.get(&user_addr) {
                if let Err(e) = tx.send(ws_message.clone()) {
                    warn!("Failed to send message to {}: {}", user_addr, e);
                }
            }
        }
    }

    /// Send a message to a specific client
    async fn send_to_client(&self, addr: SocketAddr, message: BinaryMessage) -> Result<(), String> {
        let encoded = message.encode();
        let ws_message = Message::Binary(encoded.into());

        let connections = self.connections.read().await;
        if let Some(tx) = connections.get(&addr) {
            tx.send(ws_message)
                .map_err(|e| format!("Send error: {}", e))?;
            Ok(())
        } else {
            Err(format!("Client {} not found", addr))
        }
    }

    /// Generate a random cursor color (RGB)
    fn generate_color() -> [u8; 3] {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate vibrant colors by ensuring at least one channel is high
        // and avoiding too-dark or too-light colors
        let strategy = rng.gen_range(0..3);

        match strategy {
            0 => {
                // Red dominant
                [
                    rng.gen_range(180..=255),
                    rng.gen_range(0..=150),
                    rng.gen_range(0..=150),
                ]
            }
            1 => {
                // Green dominant
                [
                    rng.gen_range(0..=150),
                    rng.gen_range(180..=255),
                    rng.gen_range(0..=150),
                ]
            }
            _ => {
                // Blue dominant
                [
                    rng.gen_range(0..=150),
                    rng.gen_range(0..=150),
                    rng.gen_range(180..=255),
                ]
            }
        }
    }

    /// Get current user count for a board (for testing/debugging)
    #[allow(dead_code)]
    pub async fn get_room_user_count(&self, board_id: u16) -> usize {
        let rooms = self.rooms.read().await;
        rooms.get(&board_id).map(|r| r.user_count()).unwrap_or(0)
    }

    /// Get current room count (for testing/debugging)
    #[allow(dead_code)]
    pub async fn get_room_count(&self) -> usize {
        let rooms = self.rooms.read().await;
        rooms.len()
    }
}

// Note: Default trait removed because ConnectionManager now requires Redis

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_color_generation() {
        // Test that colors are vibrant (at least one channel is high)
        for _ in 0..100 {
            let color = ConnectionManager::generate_color();
            let max_channel = color.iter().max().unwrap();
            assert!(
                *max_channel >= 180,
                "Color should have at least one vibrant channel"
            );
        }
    }

    // Note: test_manager_creation removed - requires Redis client for initialization
}
