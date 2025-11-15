//! Redis pub/sub implementation for broadcasting presence messages across instances.

use crate::protocol::messages::BinaryMessage;
use crate::redis::client::{RedisClient, RedisError};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Wrapper for Redis messages with instance ID to prevent echo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisMessage {
    /// Unique identifier for the service instance that sent this message
    pub instance_id: String,
    /// Encoded binary message payload
    pub payload: Vec<u8>,
}

impl RedisMessage {
    /// Create a new Redis message with instance ID
    pub fn new(instance_id: String, message: &BinaryMessage) -> Self {
        Self {
            instance_id,
            payload: message.encode(),
        }
    }

    /// Encode the Redis message to JSON for transmission
    pub fn encode(&self) -> Result<Vec<u8>, RedisError> {
        serde_json::to_vec(self).map_err(|e| {
            RedisError::ConnectionError(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "JSON encode error",
                e.to_string(),
            )))
        })
    }

    /// Decode a Redis message from JSON
    pub fn decode(data: &[u8]) -> Result<Self, RedisError> {
        serde_json::from_slice(data).map_err(|e| {
            RedisError::ConnectionError(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "JSON decode error",
                e.to_string(),
            )))
        })
    }

    /// Get the binary message from the payload
    pub fn get_binary_message(
        &self,
    ) -> Result<BinaryMessage, crate::protocol::messages::ProtocolError> {
        BinaryMessage::decode(&self.payload)
    }
}

/// Stream of incoming Redis pub/sub messages
pub type PubSubStream = mpsc::UnboundedReceiver<(String, RedisMessage)>;

/// Redis pub/sub manager for broadcasting presence updates
#[derive(Clone)]
pub struct RedisPubSub {
    client: RedisClient,
}

impl RedisPubSub {
    /// Create a new Redis pub/sub manager
    ///
    /// # Arguments
    ///
    /// * `client` - The Redis client to use for pub/sub operations
    ///
    /// # Returns
    ///
    /// A `Result` containing the `RedisPubSub` instance or a `RedisError`
    pub async fn new(client: RedisClient) -> Result<Self, RedisError> {
        info!("Initializing Redis pub/sub");
        Ok(Self { client })
    }

    /// Publish a message to a Redis channel
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel name to publish to
    /// * `message` - The message bytes to publish
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `RedisError`
    pub async fn publish(&self, channel: &str, message: &[u8]) -> Result<(), RedisError> {
        use redis::AsyncCommands;

        let mut conn = self.client.get_connection().await?;

        conn.publish::<_, _, ()>(channel, message)
            .await
            .map_err(|e| {
                warn!("Failed to publish to channel {}: {}", channel, e);
                RedisError::ConnectionError(e)
            })?;

        debug!("Published {} bytes to channel {}", message.len(), channel);
        Ok(())
    }

    /// Subscribe to Redis channels and return a stream of messages
    ///
    /// # Arguments
    ///
    /// * `channels` - List of channel names to subscribe to
    ///
    /// # Returns
    ///
    /// A `Result` containing a stream of (channel, message) tuples or a `RedisError`
    pub async fn subscribe(&self, channels: Vec<String>) -> Result<PubSubStream, RedisError> {
        info!("Subscribing to channels: {:?}", channels);

        // Get a dedicated connection for pub/sub
        let mut pubsub = self.client.client().get_async_pubsub().await?;

        // Subscribe to all channels
        for channel in &channels {
            pubsub.subscribe(channel).await?;
        }

        info!("Successfully subscribed to {} channels", channels.len());

        // Create a channel for streaming messages to the caller
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn a task to handle incoming messages
        tokio::spawn(async move {
            let mut stream = pubsub.on_message();

            while let Some(msg) = stream.next().await {
                let channel = msg.get_channel_name().to_string();
                let payload: Vec<u8> = match msg.get_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to get message payload: {}", e);
                        continue;
                    }
                };

                // Decode Redis message
                let redis_msg = match RedisMessage::decode(&payload) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to decode Redis message: {}", e);
                        continue;
                    }
                };

                // Send to channel
                if let Err(e) = tx.send((channel.clone(), redis_msg)) {
                    error!("Failed to send message to channel {}: {}", channel, e);
                    break;
                }
            }

            info!("Pub/sub stream ended");
        });

        Ok(rx)
    }

    /// Get the channel name for a specific board
    pub fn board_channel(board_id: u16) -> String {
        format!("presence:board:{}", board_id)
    }

    /// Get the global presence channel name
    pub fn global_channel() -> String {
        "presence:global".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_message_encoding() {
        let msg = BinaryMessage::Heartbeat;
        let redis_msg = RedisMessage::new("test-instance".to_string(), &msg);

        let encoded = redis_msg.encode().unwrap();
        let decoded = RedisMessage::decode(&encoded).unwrap();

        assert_eq!(redis_msg.instance_id, decoded.instance_id);
        assert_eq!(redis_msg.payload, decoded.payload);
    }

    #[test]
    fn test_redis_message_binary_decode() {
        let msg = BinaryMessage::Heartbeat;
        let redis_msg = RedisMessage::new("test-instance".to_string(), &msg);

        let binary_msg = redis_msg.get_binary_message().unwrap();
        assert_eq!(binary_msg, msg);
    }

    #[test]
    fn test_channel_names() {
        assert_eq!(RedisPubSub::board_channel(123), "presence:board:123");
        assert_eq!(RedisPubSub::global_channel(), "presence:global");
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_publish_subscribe() {
        let redis_url = "redis://localhost:6379";
        let client = RedisClient::new(redis_url).await.unwrap();
        let pubsub = RedisPubSub::new(client.clone()).await.unwrap();

        // Subscribe to a test channel
        let mut stream = pubsub
            .subscribe(vec!["test:channel".to_string()])
            .await
            .unwrap();

        // Publish a message
        let msg = BinaryMessage::Heartbeat;
        let redis_msg = RedisMessage::new("test-instance".to_string(), &msg);
        let encoded = redis_msg.encode().unwrap();

        pubsub.publish("test:channel", &encoded).await.unwrap();

        // Receive the message
        let (channel, received_msg) =
            tokio::time::timeout(std::time::Duration::from_secs(1), stream.recv())
                .await
                .unwrap()
                .unwrap();

        assert_eq!(channel, "test:channel");
        assert_eq!(received_msg.instance_id, "test-instance");
    }
}
