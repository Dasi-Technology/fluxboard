//! Binary protocol implementation for WebSocket presence system.
//!
//! This module implements a highly efficient binary protocol that achieves:
//! - 5-6 byte cursor updates (vs ~130 bytes JSON = 96% reduction)
//! - Big-endian byte order for all multi-byte integers
//! - Type-safe encoding/decoding with comprehensive error handling
//! - Zero-copy parsing where possible

pub mod codec;
pub mod messages;
pub mod types;

pub use codec::BinaryCodec;
pub use messages::{denormalize_coord, normalize_coord, BinaryMessage, ProtocolError};
pub use types::*;
