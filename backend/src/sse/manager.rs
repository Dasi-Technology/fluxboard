use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use super::events::SseEvent;

/// Maximum number of events to buffer per client
const CHANNEL_BUFFER_SIZE: usize = 100;

/// SSE Event wrapper that can be formatted for streaming
#[derive(Clone)]
pub struct SseEventWrapper {
    event: SseEvent,
}

impl SseEventWrapper {
    pub fn new(event: SseEvent) -> Self {
        Self { event }
    }
}

impl fmt::Display for SseEventWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.event.to_json() {
            Ok(json) => {
                write!(f, "event: {}\ndata: {}\n\n", self.event.event_name(), json)
            }
            Err(e) => {
                log::error!("Failed to serialize SSE event: {}", e);
                write!(f, ": error\n\n")
            }
        }
    }
}

/// Manager for SSE connections with per-board client tracking
#[derive(Clone)]
pub struct SseManager {
    /// Map of board_id -> list of client channels
    /// Each client has a channel sender to receive events
    connections: Arc<RwLock<HashMap<Uuid, Vec<mpsc::Sender<Result<SseEventWrapper, Infallible>>>>>>,
}

impl SseManager {
    /// Create a new SSE manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to updates for a specific board
    /// Returns a receiver that will get events for this board
    pub async fn subscribe(
        &self,
        board_id: Uuid,
    ) -> mpsc::Receiver<Result<SseEventWrapper, Infallible>> {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

        let mut connections = self.connections.write().await;
        connections
            .entry(board_id)
            .or_insert_with(Vec::new)
            .push(tx);

        rx
    }

    /// Broadcast an event to all clients subscribed to a board
    pub async fn broadcast(&self, board_id: Uuid, event: SseEvent) {
        let wrapped_event = Ok(SseEventWrapper::new(event));

        let mut connections = self.connections.write().await;

        if let Some(clients) = connections.get_mut(&board_id) {
            // Send to all clients, removing any that have disconnected
            clients.retain(|client| {
                // Try to send, if it fails the client has disconnected
                match client.try_send(wrapped_event.clone()) {
                    Ok(_) => true,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        // Channel is full, keep the client but log warning
                        log::warn!("SSE client channel is full for board {}", board_id);
                        true
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        // Client disconnected, remove it
                        log::debug!("Removing disconnected SSE client for board {}", board_id);
                        false
                    }
                }
            });

            // If no clients left, remove the board entry
            if clients.is_empty() {
                connections.remove(&board_id);
                log::debug!("No more SSE clients for board {}, removing entry", board_id);
            }
        }
    }

    /// Manually cleanup closed connections for a board
    /// This is called automatically during broadcast, but can be called manually if needed
    pub async fn cleanup_closed_connections(&self, board_id: Uuid) {
        let mut connections = self.connections.write().await;

        if let Some(clients) = connections.get_mut(&board_id) {
            clients.retain(|client| !client.is_closed());

            if clients.is_empty() {
                connections.remove(&board_id);
            }
        }
    }

    /// Get the number of active connections for a board
    #[allow(dead_code)]
    pub async fn connection_count(&self, board_id: Uuid) -> usize {
        let connections = self.connections.read().await;
        connections
            .get(&board_id)
            .map(|clients| clients.len())
            .unwrap_or(0)
    }

    /// Get the total number of active connections across all boards
    #[allow(dead_code)]
    pub async fn total_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().map(|clients| clients.len()).sum()
    }
}

impl Default for SseManager {
    fn default() -> Self {
        Self::new()
    }
}
