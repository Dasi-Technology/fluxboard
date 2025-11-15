use actix::prelude::*;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::server::{Join, Leave, WsServer};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WebSocket session actor
pub struct WsSession {
    /// Unique session ID
    pub id: Uuid,
    /// Board ID this session is subscribed to
    pub board_id: Uuid,
    /// WebSocket server address
    pub server: Addr<WsServer>,
    /// Client must send ping at least once per CLIENT_TIMEOUT
    pub hb: Instant,
    /// WebSocket message sink
    pub msg_tx: actix_ws::Session,
}

impl WsSession {
    pub fn new(board_id: Uuid, server: Addr<WsServer>, msg_tx: actix_ws::Session) -> Self {
        Self {
            id: Uuid::new_v4(),
            board_id,
            server,
            hb: Instant::now(),
            msg_tx,
        }
    }

    /// Helper method to start heartbeat task
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // Heartbeat timed out
                log::warn!("WebSocket Client heartbeat failed, disconnecting!");
                act.server.do_send(Leave {
                    board_id: act.board_id,
                    session_addr: ctx.address(),
                });
                ctx.stop();
                return;
            }

            // Send ping - spawn async task
            let mut msg_tx = act.msg_tx.clone();
            actix::spawn(async move {
                let _ = msg_tx.ping(b"").await;
            });
        });
    }
}

impl Actor for WsSession {
    type Context = Context<Self>;

    /// Method is called on actor start
    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket session started: {}", self.id);

        // Start heartbeat task
        self.hb(ctx);

        // Register session with server
        self.server.do_send(Join {
            board_id: self.board_id,
            session_addr: ctx.address(),
        });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        log::info!("WebSocket session stopping: {}", self.id);

        // Notify server
        self.server.do_send(Leave {
            board_id: self.board_id,
            session_addr: ctx.address(),
        });

        Running::Stop
    }
}

/// Message to send to client
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub message: String,
}

/// Handler for SendMessage
impl Handler<SendMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let mut msg_tx = self.msg_tx.clone();
        actix::spawn(async move {
            if msg_tx.text(msg.message).await.is_err() {
                log::error!("Failed to send message to WebSocket client");
            }
        });
    }
}

/// Message to handle incoming WebSocket messages
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub message: String,
}

/// Handler for ClientMessage
impl Handler<ClientMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Context<Self>) -> Self::Result {
        log::debug!("Received message from client: {}", msg.message);

        // Parse and handle client messages if needed
        // For now, we just log them as the updates come from HTTP handlers
    }
}
