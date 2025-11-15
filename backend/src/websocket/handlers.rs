use actix::Addr;
use actix_web::{HttpRequest, HttpResponse, web};
use sqlx::PgPool;

use super::server::WsServer;
use crate::error::AppResult;
use crate::models::Board;

/// WebSocket handler endpoint
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<WsServer>>,
    pool: web::Data<PgPool>,
    share_token: web::Path<String>,
) -> AppResult<HttpResponse> {
    // Validate share token and get board
    let board = Board::find_by_share_token(pool.get_ref(), &share_token.into_inner())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Invalid share token".to_string()))?;

    log::info!("WebSocket connection request for board: {}", board.id);

    // Perform WebSocket upgrade
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn WebSocket session actor
    actix_web::rt::spawn(async move {
        let mut session_actor =
            super::session::WsSession::new(board.id, server.get_ref().clone(), session);

        // Handle incoming messages
        let mut msg_stream = msg_stream;
        while let Some(Ok(msg)) = futures_util::StreamExt::next(&mut msg_stream).await {
            match msg {
                actix_ws::Message::Ping(bytes) => {
                    session_actor.hb = std::time::Instant::now();
                    let _ = session_actor.msg_tx.pong(&bytes).await;
                }
                actix_ws::Message::Pong(_) => {
                    session_actor.hb = std::time::Instant::now();
                }
                actix_ws::Message::Text(text) => {
                    log::debug!("Received text message: {}", text);
                    session_actor.hb = std::time::Instant::now();
                }
                actix_ws::Message::Close(reason) => {
                    log::info!("WebSocket closed: {:?}", reason);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(response)
}
