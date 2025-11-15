//! WebSocket module
//!
//! This module handles real-time WebSocket communication for collaborative features.
//! It provides message types, session management, and server coordination.

pub mod handlers;
pub mod messages;
pub mod server;
pub mod session;

// Re-export key types
pub use handlers::ws_handler;
pub use server::WsServer;
