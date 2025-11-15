use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::messages::WsMessage;

/// WebSocket server actor that manages all active connections
pub struct WsServer {
    /// Map of board_id to set of session IDs
    rooms: HashMap<Uuid, HashSet<Addr<super::session::WsSession>>>,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

/// Message to join a board room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub board_id: Uuid,
    pub session_addr: Addr<super::session::WsSession>,
}

/// Message to leave a board room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub board_id: Uuid,
    pub session_addr: Addr<super::session::WsSession>,
}

/// Message to broadcast to all sessions in a board
#[derive(Message)]
#[rtype(result = "()")]
pub struct Broadcast {
    pub message: WsMessage,
}

/// Handler for Join messages
impl Handler<Join> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _ctx: &mut Context<Self>) -> Self::Result {
        log::info!("Session joining board room: {}", msg.board_id);

        self.rooms
            .entry(msg.board_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_addr);
    }
}

/// Handler for Leave messages
impl Handler<Leave> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, _ctx: &mut Context<Self>) -> Self::Result {
        log::info!("Session leaving board room: {}", msg.board_id);

        if let Some(sessions) = self.rooms.get_mut(&msg.board_id) {
            sessions.remove(&msg.session_addr);

            // Remove room if empty
            if sessions.is_empty() {
                self.rooms.remove(&msg.board_id);
            }
        }
    }
}

/// Handler for Broadcast messages
impl Handler<Broadcast> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Broadcast, _ctx: &mut Context<Self>) -> Self::Result {
        let board_id = msg.message.board_id;

        if let Some(sessions) = self.rooms.get(&board_id) {
            let message_json = match msg.message.to_json() {
                Ok(json) => json,
                Err(e) => {
                    log::error!("Failed to serialize WebSocket message: {}", e);
                    return;
                }
            };

            log::debug!(
                "Broadcasting message to {} sessions in board {}",
                sessions.len(),
                board_id
            );

            for session in sessions {
                session.do_send(super::session::SendMessage {
                    message: message_json.clone(),
                });
            }
        }
    }
}
