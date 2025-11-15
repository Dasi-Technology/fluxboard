//! Binary protocol message type constants.
//!
//! Each message type is identified by a single byte that appears as the first byte
//! of every encoded message. This allows for efficient message routing and parsing.

/// Client → Server: Cursor position update (5 bytes total)
pub const MSG_CURSOR_UPDATE: u8 = 0x01;

/// Server → Client: Broadcast cursor position to other users (7 bytes total)
pub const MSG_CURSOR_BROADCAST: u8 = 0x02;

/// Client → Server: Join a board (4-36 bytes total)
pub const MSG_JOIN: u8 = 0x03;

/// Client → Server: Leave a board (3 bytes total)
pub const MSG_LEAVE: u8 = 0x04;

/// Server → Client: Notify that a user joined (7-40 bytes total)
pub const MSG_USER_JOINED: u8 = 0x05;

/// Server → Client: Notify that a user left (4 bytes total)
pub const MSG_USER_LEFT: u8 = 0x06;

/// Server → Client: Update presence count for a board (4 bytes total)
pub const MSG_PRESENCE_UPDATE: u8 = 0x07;

/// Bidirectional: Heartbeat/keepalive (1 byte total)
pub const MSG_HEARTBEAT: u8 = 0x08;

/// Maximum username length in bytes (UTF-8 encoded)
pub const MAX_USERNAME_LENGTH: usize = 32;
