//! WebSocket frame codec for binary protocol messages.
//!
//! This module provides a codec wrapper around the binary message encoding/decoding
//! to integrate with WebSocket frame handling.

use super::messages::{BinaryMessage, ProtocolError};
use bytes::Bytes;

/// A codec for encoding and decoding binary protocol messages in WebSocket frames.
///
/// This codec wraps the `BinaryMessage` encoding/decoding logic and provides
/// a simple interface for use with WebSocket connections.
#[derive(Debug, Clone, Default)]
pub struct BinaryCodec;

impl BinaryCodec {
    /// Create a new binary codec.
    pub fn new() -> Self {
        Self
    }

    /// Encode a binary message into bytes suitable for sending over a WebSocket.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to encode
    ///
    /// # Returns
    ///
    /// A `Bytes` buffer containing the encoded message
    pub fn encode(&self, message: &BinaryMessage) -> Bytes {
        Bytes::from(message.encode())
    }

    /// Decode bytes from a WebSocket frame into a binary message.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte buffer to decode
    ///
    /// # Returns
    ///
    /// A `Result` containing the decoded message or a `ProtocolError`
    pub fn decode(&self, data: &[u8]) -> Result<BinaryMessage, ProtocolError> {
        BinaryMessage::decode(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::messages::normalize_coord;

    #[test]
    fn test_codec_roundtrip() {
        let codec = BinaryCodec::new();

        let original = BinaryMessage::CursorUpdate {
            board_id: 42,
            x: normalize_coord(0.3),
            y: normalize_coord(0.7),
        };

        let encoded = codec.encode(&original);
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_codec_heartbeat() {
        let codec = BinaryCodec::new();

        let original = BinaryMessage::Heartbeat;
        let encoded = codec.encode(&original);

        assert_eq!(encoded.len(), 1);

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_codec_join_message() {
        let codec = BinaryCodec::new();

        let original = BinaryMessage::Join {
            board_id: 100,
            username: "Alice".to_string(),
        };

        let encoded = codec.encode(&original);
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_codec_user_joined_message() {
        let codec = BinaryCodec::new();

        let original = BinaryMessage::UserJoined {
            board_id: 200,
            user_id: 5,
            username: "Bob".to_string(),
            color: [255, 128, 64],
        };

        let encoded = codec.encode(&original);
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(original, decoded);
    }
}
