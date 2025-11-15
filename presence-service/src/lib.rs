//! Presence service library.
//!
//! This library provides the core functionality for the WebSocket-based
//! real-time presence and cursor tracking system.

pub mod config;
pub mod connection;
pub mod handlers;
pub mod presence;
pub mod protocol;
pub mod redis;
pub mod utils;

pub use protocol::{denormalize_coord, normalize_coord, BinaryCodec, BinaryMessage, ProtocolError};
