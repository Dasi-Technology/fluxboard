//! Server-Sent Events (SSE) module
//!
//! This module handles real-time server-to-client communication via SSE.
//! It provides event types, connection management, and broadcasting capabilities.

pub mod events;
pub mod manager;

// Re-export key types
pub use manager::SseManager;
