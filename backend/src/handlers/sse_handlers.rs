use actix_web::{Error, HttpRequest, HttpResponse, web};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::ReceiverStream;

use crate::error::AppError;
use crate::sse::SseManager;
use sqlx::PgPool;

/// SSE endpoint for board updates
/// GET /sse/{share_token}
pub async fn board_events_stream(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    path: web::Path<String>,
    _req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let share_token = path.into_inner();

    // Validate share_token and get board_id
    let board = sqlx::query!(
        r#"
        SELECT id, share_token, created_at, updated_at
        FROM boards
        WHERE share_token = $1
        "#,
        share_token
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error fetching board: {}", e);
        AppError::DatabaseError(e)
    })?
    .ok_or_else(|| {
        log::warn!("Board not found for share_token: {}", share_token);
        AppError::NotFound("Board not found".to_string())
    })?;

    let board_id = board.id;

    log::info!("New SSE connection for board: {}", board_id);

    // Subscribe to board updates
    let receiver = sse_manager.subscribe(board_id).await;
    let event_stream = ReceiverStream::new(receiver);

    // Create a heartbeat stream that sends keep-alive comments every 30 seconds
    let heartbeat = stream::repeat_with(|| {
        Ok::<actix_web::web::Bytes, Infallible>(actix_web::web::Bytes::from(": keep-alive\n\n"))
    })
    .throttle(Duration::from_secs(30));

    // Convert SSE events to Bytes
    let event_bytes_stream = event_stream.map(|event_result| {
        match event_result {
            Ok(event) => {
                // Format: "event: {name}\ndata: {data}\n\n"
                let formatted = format!("{}", event);
                Ok::<actix_web::web::Bytes, Infallible>(actix_web::web::Bytes::from(formatted))
            }
            Err(_) => {
                // This should never happen with Infallible
                Ok(actix_web::web::Bytes::from(": error\n\n"))
            }
        }
    });

    // Merge the event stream with the heartbeat stream
    let merged_stream: Pin<
        Box<dyn Stream<Item = Result<actix_web::web::Bytes, Infallible>> + Send>,
    > = Box::pin(stream::select(event_bytes_stream, heartbeat));

    // Create the SSE response with proper headers
    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(merged_stream))
}
