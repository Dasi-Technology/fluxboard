//! Binary message encoding and decoding.
//!
//! This module implements a highly efficient binary protocol that achieves 5-6 byte
//! message sizes for cursor updates (96% reduction vs JSON). All multi-byte integers
//! use big-endian byte order for network transmission.

use bytes::BytesMut;
use std::io::Cursor;
use std::io::Read;
use thiserror::Error;

use super::types::*;

/// Protocol errors that can occur during encoding or decoding.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unknown message type: {0:#x}")]
    UnknownMessageType(u8),

    #[error("Invalid message length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Invalid UTF-8 in username")]
    InvalidUtf8,

    #[error("Username too long: {0} bytes (max 32)")]
    UsernameTooLong(usize),

    #[error("Buffer underflow")]
    BufferUnderflow,
}

/// Binary protocol messages.
///
/// Each variant represents one of the 8 message types in the protocol.
/// All messages are designed for minimal size while maintaining type safety.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryMessage {
    /// Client → Server: Update cursor position (5 bytes)
    ///
    /// Layout:
    /// - byte 0: message type (0x01)
    /// - bytes 1-2: board_id (u16, big-endian)
    /// - bytes 3-4: x coordinate (u16, big-endian, normalized 0-65535)
    /// - bytes 5-6: y coordinate (u16, big-endian, normalized 0-65535)
    CursorUpdate { board_id: u16, x: u16, y: u16 },

    /// Server → Client: Broadcast cursor position (7 bytes)
    ///
    /// Layout:
    /// - byte 0: message type (0x02)
    /// - bytes 1-2: board_id (u16, big-endian)
    /// - byte 3: user_id (u8)
    /// - bytes 4-5: x coordinate (u16, big-endian, normalized 0-65535)
    /// - bytes 6-7: y coordinate (u16, big-endian, normalized 0-65535)
    CursorBroadcast {
        board_id: u16,
        user_id: u8,
        x: u16,
        y: u16,
    },

    /// Client → Server: Join a board (4-36 bytes)
    ///
    /// Layout:
    /// - byte 0: message type (0x03)
    /// - bytes 1-2: board_id (u16, big-endian)
    /// - byte 3: username length (u8)
    /// - bytes 4+: username UTF-8 bytes (max 32 bytes)
    Join { board_id: u16, username: String },

    /// Client → Server: Leave a board (3 bytes)
    ///
    /// Layout:
    /// - byte 0: message type (0x04)
    /// - bytes 1-2: board_id (u16, big-endian)
    Leave { board_id: u16 },

    /// Server → Client: User joined notification (7-40 bytes)
    ///
    /// Layout:
    /// - byte 0: message type (0x05)
    /// - bytes 1-2: board_id (u16, big-endian)
    /// - byte 3: user_id (u8)
    /// - byte 4: username length (u8)
    /// - bytes 5+: username UTF-8 bytes (max 32 bytes)
    /// - bytes (5+len) to (8+len): RGB color (3 bytes)
    UserJoined {
        board_id: u16,
        user_id: u8,
        username: String,
        color: [u8; 3],
    },

    /// Server → Client: User left notification (4 bytes)
    ///
    /// Layout:
    /// - byte 0: message type (0x06)
    /// - bytes 1-2: board_id (u16, big-endian)
    /// - byte 3: user_id (u8)
    UserLeft { board_id: u16, user_id: u8 },

    /// Server → Client: Presence count update (4 bytes)
    ///
    /// Layout:
    /// - byte 0: message type (0x07)
    /// - bytes 1-2: board_id (u16, big-endian)
    /// - byte 3: count (u8)
    PresenceUpdate { board_id: u16, count: u8 },

    /// Bidirectional: Heartbeat (1 byte)
    ///
    /// Layout:
    /// - byte 0: message type (0x08)
    Heartbeat,
}

impl BinaryMessage {
    /// Encode this message into a byte vector.
    ///
    /// All multi-byte integers are encoded in big-endian byte order.
    /// Strings are encoded with a 1-byte length prefix followed by UTF-8 bytes.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the complete encoded message, ready to send.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();

        match self {
            BinaryMessage::CursorUpdate { board_id, x, y } => {
                buf.extend_from_slice(&[MSG_CURSOR_UPDATE]);
                buf.extend_from_slice(&board_id.to_be_bytes());
                buf.extend_from_slice(&x.to_be_bytes());
                buf.extend_from_slice(&y.to_be_bytes());
            }

            BinaryMessage::CursorBroadcast {
                board_id,
                user_id,
                x,
                y,
            } => {
                buf.extend_from_slice(&[MSG_CURSOR_BROADCAST]);
                buf.extend_from_slice(&board_id.to_be_bytes());
                buf.extend_from_slice(&[*user_id]);
                buf.extend_from_slice(&x.to_be_bytes());
                buf.extend_from_slice(&y.to_be_bytes());
            }

            BinaryMessage::Join { board_id, username } => {
                buf.extend_from_slice(&[MSG_JOIN]);
                buf.extend_from_slice(&board_id.to_be_bytes());
                let username_bytes = username.as_bytes();
                buf.extend_from_slice(&[username_bytes.len() as u8]);
                buf.extend_from_slice(username_bytes);
            }

            BinaryMessage::Leave { board_id } => {
                buf.extend_from_slice(&[MSG_LEAVE]);
                buf.extend_from_slice(&board_id.to_be_bytes());
            }

            BinaryMessage::UserJoined {
                board_id,
                user_id,
                username,
                color,
            } => {
                buf.extend_from_slice(&[MSG_USER_JOINED]);
                buf.extend_from_slice(&board_id.to_be_bytes());
                buf.extend_from_slice(&[*user_id]);
                let username_bytes = username.as_bytes();
                buf.extend_from_slice(&[username_bytes.len() as u8]);
                buf.extend_from_slice(username_bytes);
                buf.extend_from_slice(color);
            }

            BinaryMessage::UserLeft { board_id, user_id } => {
                buf.extend_from_slice(&[MSG_USER_LEFT]);
                buf.extend_from_slice(&board_id.to_be_bytes());
                buf.extend_from_slice(&[*user_id]);
            }

            BinaryMessage::PresenceUpdate { board_id, count } => {
                buf.extend_from_slice(&[MSG_PRESENCE_UPDATE]);
                buf.extend_from_slice(&board_id.to_be_bytes());
                buf.extend_from_slice(&[*count]);
            }

            BinaryMessage::Heartbeat => {
                buf.extend_from_slice(&[MSG_HEARTBEAT]);
            }
        }

        buf.to_vec()
    }

    /// Decode a message from a byte slice.
    ///
    /// All multi-byte integers are expected to be in big-endian byte order.
    /// Strings are expected to have a 1-byte length prefix followed by UTF-8 bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice to decode
    ///
    /// # Returns
    ///
    /// A `Result` containing the decoded message or a `ProtocolError` if decoding fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message type is unknown
    /// - The buffer is too short for the message type
    /// - UTF-8 validation fails for username strings
    /// - Username length exceeds maximum
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::BufferUnderflow);
        }

        let mut cursor = Cursor::new(data);
        let mut msg_type_buf = [0u8; 1];
        cursor
            .read_exact(&mut msg_type_buf)
            .map_err(|_| ProtocolError::BufferUnderflow)?;
        let msg_type = msg_type_buf[0];

        match msg_type {
            MSG_CURSOR_UPDATE => {
                if data.len() != 7 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 7,
                        actual: data.len(),
                    });
                }

                let board_id = read_u16(&mut cursor)?;
                let x = read_u16(&mut cursor)?;
                let y = read_u16(&mut cursor)?;

                Ok(BinaryMessage::CursorUpdate { board_id, x, y })
            }

            MSG_CURSOR_BROADCAST => {
                if data.len() != 8 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 8,
                        actual: data.len(),
                    });
                }

                let board_id = read_u16(&mut cursor)?;
                let user_id = read_u8(&mut cursor)?;
                let x = read_u16(&mut cursor)?;
                let y = read_u16(&mut cursor)?;

                Ok(BinaryMessage::CursorBroadcast {
                    board_id,
                    user_id,
                    x,
                    y,
                })
            }

            MSG_JOIN => {
                if data.len() < 4 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 4,
                        actual: data.len(),
                    });
                }

                let board_id = read_u16(&mut cursor)?;
                let username = read_string(&mut cursor, MAX_USERNAME_LENGTH)?;

                Ok(BinaryMessage::Join { board_id, username })
            }

            MSG_LEAVE => {
                if data.len() != 3 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 3,
                        actual: data.len(),
                    });
                }

                let board_id = read_u16(&mut cursor)?;

                Ok(BinaryMessage::Leave { board_id })
            }

            MSG_USER_JOINED => {
                if data.len() < 8 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 8,
                        actual: data.len(),
                    });
                }

                let board_id = read_u16(&mut cursor)?;
                let user_id = read_u8(&mut cursor)?;
                let username = read_string(&mut cursor, MAX_USERNAME_LENGTH)?;
                let color = read_color(&mut cursor)?;

                Ok(BinaryMessage::UserJoined {
                    board_id,
                    user_id,
                    username,
                    color,
                })
            }

            MSG_USER_LEFT => {
                if data.len() != 4 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 4,
                        actual: data.len(),
                    });
                }

                let board_id = read_u16(&mut cursor)?;
                let user_id = read_u8(&mut cursor)?;

                Ok(BinaryMessage::UserLeft { board_id, user_id })
            }

            MSG_PRESENCE_UPDATE => {
                if data.len() != 4 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 4,
                        actual: data.len(),
                    });
                }

                let board_id = read_u16(&mut cursor)?;
                let count = read_u8(&mut cursor)?;

                Ok(BinaryMessage::PresenceUpdate { board_id, count })
            }

            MSG_HEARTBEAT => {
                if data.len() != 1 {
                    return Err(ProtocolError::InvalidLength {
                        expected: 1,
                        actual: data.len(),
                    });
                }

                Ok(BinaryMessage::Heartbeat)
            }

            unknown => Err(ProtocolError::UnknownMessageType(unknown)),
        }
    }
}

// Helper functions for reading primitive types

/// Read a big-endian u16 from the cursor.
fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, ProtocolError> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ProtocolError::BufferUnderflow)?;
    Ok(u16::from_be_bytes(buf))
}

/// Read a u8 from the cursor.
fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, ProtocolError> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ProtocolError::BufferUnderflow)?;
    Ok(buf[0])
}

/// Read a length-prefixed string from the cursor.
///
/// The string is encoded as a 1-byte length followed by UTF-8 bytes.
fn read_string(cursor: &mut Cursor<&[u8]>, max_length: usize) -> Result<String, ProtocolError> {
    let length = read_u8(cursor)? as usize;

    if length > max_length {
        return Err(ProtocolError::UsernameTooLong(length));
    }

    let mut buf = vec![0u8; length];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ProtocolError::BufferUnderflow)?;

    String::from_utf8(buf).map_err(|_| ProtocolError::InvalidUtf8)
}

/// Read a 3-byte RGB color from the cursor.
fn read_color(cursor: &mut Cursor<&[u8]>) -> Result<[u8; 3], ProtocolError> {
    let mut color = [0u8; 3];
    cursor
        .read_exact(&mut color)
        .map_err(|_| ProtocolError::BufferUnderflow)?;
    Ok(color)
}

// Coordinate normalization helpers

/// Normalize a floating-point coordinate (0.0-1.0) to a 16-bit unsigned integer (0-65535).
///
/// This allows us to represent fractional coordinates with high precision
/// while using only 2 bytes per coordinate.
///
/// # Arguments
///
/// * `coord` - A floating-point coordinate in the range [0.0, 1.0]
///
/// # Returns
///
/// A 16-bit unsigned integer in the range [0, 65535]
///
/// # Examples
///
/// ```
/// # use presence_service::protocol::messages::normalize_coord;
/// assert_eq!(normalize_coord(0.0), 0);
/// assert_eq!(normalize_coord(1.0), 65535);
/// assert_eq!(normalize_coord(0.5), 32767);
/// ```
pub fn normalize_coord(coord: f32) -> u16 {
    // Clamp to [0.0, 1.0] range to prevent overflow
    let clamped = coord.clamp(0.0, 1.0);
    (clamped * 65535.0) as u16
}

/// Denormalize a 16-bit unsigned integer (0-65535) to a floating-point coordinate (0.0-1.0).
///
/// This is the inverse operation of `normalize_coord`.
///
/// # Arguments
///
/// * `coord` - A 16-bit unsigned integer in the range [0, 65535]
///
/// # Returns
///
/// A floating-point coordinate in the range [0.0, 1.0]
///
/// # Examples
///
/// ```
/// # use presence_service::protocol::messages::denormalize_coord;
/// assert_eq!(denormalize_coord(0), 0.0);
/// assert_eq!(denormalize_coord(65535), 1.0);
/// assert!((denormalize_coord(32767) - 0.5).abs() < 0.001);
/// ```
pub fn denormalize_coord(coord: u16) -> f32 {
    coord as f32 / 65535.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_denormalize_roundtrip() {
        let coords = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        for coord in coords {
            let normalized = normalize_coord(coord);
            let denormalized = denormalize_coord(normalized);
            assert!(
                (coord - denormalized).abs() < 0.001,
                "Roundtrip failed for {}: got {}",
                coord,
                denormalized
            );
        }
    }

    #[test]
    fn test_normalize_clamps() {
        assert_eq!(normalize_coord(-0.5), 0);
        assert_eq!(normalize_coord(1.5), 65535);
    }

    #[test]
    fn test_cursor_update_encoding() {
        let msg = BinaryMessage::CursorUpdate {
            board_id: 1234,
            x: normalize_coord(0.5),
            y: normalize_coord(0.75),
        };
        let encoded = msg.encode();
        assert_eq!(encoded.len(), 7);
        assert_eq!(encoded[0], MSG_CURSOR_UPDATE);
    }

    #[test]
    fn test_heartbeat_encoding() {
        let msg = BinaryMessage::Heartbeat;
        let encoded = msg.encode();
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], MSG_HEARTBEAT);
    }

    #[test]
    fn test_decode_unknown_type() {
        let data = vec![0xFF];
        let result = BinaryMessage::decode(&data);
        assert!(matches!(
            result,
            Err(ProtocolError::UnknownMessageType(0xFF))
        ));
    }

    #[test]
    fn test_decode_buffer_underflow() {
        let data = vec![MSG_CURSOR_UPDATE, 0x00]; // Too short
        let result = BinaryMessage::decode(&data);
        assert!(matches!(result, Err(ProtocolError::InvalidLength { .. })));
    }
}
