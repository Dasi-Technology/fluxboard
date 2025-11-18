use actix_web::{HttpRequest, HttpResponse, web};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::auth_middleware::auth::AuthenticatedUser;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{Board, Card, CardAttachment, Column, UploadUrlRequest, UploadUrlResponse};
use crate::services::S3Service;
use crate::sse::events::SseEvent;
use crate::sse::manager::SseManager;

/// Helper function to check if a board operation is allowed
fn is_board_operation_allowed(board: &Board, req: &HttpRequest) -> bool {
    if !board.is_locked {
        return true;
    }

    if let Some(password_header) = req.headers().get("X-Board-Password") {
        if let Ok(password_str) = password_header.to_str() {
            return password_str == board.password;
        }
    }

    false
}

/// Helper function to get board from card_id
async fn get_board_from_card(pool: &PgPool, card_id: Uuid) -> AppResult<Board> {
    let card = Card::find_by_id(pool, card_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Card not found".to_string()))?;

    let column = Column::find_by_id(pool, card.column_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool, column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    Ok(board)
}

/// Generate a pre-signed upload URL for a card attachment
pub async fn generate_upload_url(
    pool: web::Data<PgPool>,
    s3_service: web::Data<Arc<S3Service>>,
    config: web::Data<Config>,
    card_id: web::Path<Uuid>,
    input: web::Json<UploadUrlRequest>,
    user: AuthenticatedUser,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let card_id = card_id.into_inner();
    let input = input.into_inner();

    // Validate input
    input.validate()?;

    // Check if card exists and get board for password verification
    let board = get_board_from_card(pool.get_ref(), card_id).await?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot upload attachments to a locked board".to_string(),
        ));
    }

    // Validate file size
    if input.file_size > config.s3_upload_max_size as i32 {
        return Err(AppError::BadRequest(format!(
            "File size exceeds maximum allowed size of {} bytes",
            config.s3_upload_max_size
        )));
    }

    // Validate content type
    let allowed_types: Vec<&str> = config.s3_allowed_types.split(',').collect();
    if !allowed_types.contains(&input.content_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Content type '{}' is not allowed. Allowed types: {}",
            input.content_type, config.s3_allowed_types
        )));
    }

    // Extract file extension
    let extension = S3Service::extract_extension(&input.filename);

    // Generate attachment ID
    let attachment_id = Uuid::new_v4();

    log::info!(
        "[Upload URL] Generated attachment_id={} for card_id={}",
        attachment_id,
        card_id
    );

    // Build S3 key
    let s3_key = S3Service::build_s3_key(card_id, attachment_id, &extension);

    // Create attachment record (unconfirmed) with the pre-generated ID
    log::info!(
        "[Upload URL] Creating attachment record in database with id={}",
        attachment_id
    );
    let attachment = CardAttachment::create_with_id(
        pool.get_ref(),
        attachment_id,
        card_id,
        user.user_id,
        format!("{}.{}", attachment_id, extension),
        input.filename.clone(),
        input.content_type.clone(),
        input.file_size,
        s3_key.clone(),
        config.aws_s3_bucket.clone(),
    )
    .await?;

    log::info!(
        "[Upload URL] Created attachment record with id={}, is_confirmed={}",
        attachment.id,
        attachment.is_confirmed
    );

    // Generate pre-signed upload URL
    let upload_url = s3_service
        .generate_upload_url(&s3_key, &input.content_type)
        .await?;

    log::info!(
        "[Upload URL] Returning response with attachment_id={}",
        attachment_id
    );

    Ok(HttpResponse::Ok().json(UploadUrlResponse {
        upload_url,
        attachment_id,
        s3_key,
    }))
}

/// Confirm an attachment after successful upload to S3
pub async fn confirm_attachment(
    pool: web::Data<PgPool>,
    s3_service: web::Data<Arc<S3Service>>,
    sse_manager: web::Data<Arc<SseManager>>,
    path: web::Path<(Uuid, Uuid)>,
    _user: AuthenticatedUser,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let (card_id, attachment_id) = path.into_inner();

    log::info!(
        "[Confirm] Starting confirmation for card_id={}, attachment_id={}",
        card_id,
        attachment_id
    );

    // Check if card exists and get board for password verification
    let board = match get_board_from_card(pool.get_ref(), card_id).await {
        Ok(b) => {
            log::info!("[Confirm] Found board with id={}", b.id);
            b
        }
        Err(e) => {
            log::error!(
                "[Confirm] Failed to get board from card_id={}: {:?}",
                card_id,
                e
            );
            return Err(e);
        }
    };

    if !is_board_operation_allowed(&board, &req) {
        log::error!(
            "[Confirm] Board operation not allowed for board_id={}",
            board.id
        );
        return Err(AppError::Unauthorized(
            "Cannot confirm attachments on a locked board".to_string(),
        ));
    }

    // Get attachment
    log::info!("[Confirm] Looking up attachment with id={}", attachment_id);
    let attachment = CardAttachment::find_by_id(pool.get_ref(), attachment_id)
        .await?
        .ok_or_else(|| {
            log::error!("[Confirm] Attachment not found with id={}", attachment_id);
            AppError::NotFound("Attachment not found".to_string())
        })?;

    log::info!(
        "[Confirm] Found attachment: card_id={}, s3_key={}, is_confirmed={}",
        attachment.card_id,
        attachment.s3_key,
        attachment.is_confirmed
    );

    // Verify attachment belongs to the card
    if attachment.card_id != card_id {
        log::error!(
            "[Confirm] Attachment card_id mismatch: expected={}, actual={}",
            card_id,
            attachment.card_id
        );
        return Err(AppError::BadRequest(
            "Attachment does not belong to this card".to_string(),
        ));
    }

    // Verify S3 object exists
    log::info!(
        "[Confirm] Verifying S3 object exists: {}",
        attachment.s3_key
    );
    let exists = s3_service.verify_object_exists(&attachment.s3_key).await?;
    if !exists {
        log::error!("[Confirm] S3 object not found: {}", attachment.s3_key);
        return Err(AppError::BadRequest(
            "File not found in S3. Upload may have failed.".to_string(),
        ));
    }
    log::info!("[Confirm] S3 object verified");

    // Confirm attachment
    log::info!("[Confirm] Marking attachment as confirmed");
    let confirmed_attachment = CardAttachment::confirm(pool.get_ref(), attachment_id)
        .await?
        .ok_or_else(|| {
            log::error!(
                "[Confirm] Failed to confirm attachment with id={}",
                attachment_id
            );
            AppError::NotFound("Attachment not found".to_string())
        })?;

    log::info!("[Confirm] Attachment confirmed successfully");

    // Broadcast SSE event
    log::info!(
        "[Confirm] Broadcasting SSE event for board_id={}, card_id={}, attachment_id={}",
        board.id,
        card_id,
        confirmed_attachment.id
    );
    sse_manager
        .broadcast(
            board.id,
            SseEvent::AttachmentCreated {
                attachment: confirmed_attachment.clone(),
                card_id,
            },
        )
        .await;

    log::info!("[Confirm] Returning confirmed attachment response");
    Ok(HttpResponse::Ok().json(confirmed_attachment))
}

/// List all attachments for a card
pub async fn list_card_attachments(
    pool: web::Data<PgPool>,
    card_id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let card_id = card_id.into_inner();

    // Verify card exists
    Card::find_by_id(pool.get_ref(), card_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Card not found".to_string()))?;

    let attachments = CardAttachment::find_by_card_id(pool.get_ref(), card_id).await?;

    Ok(HttpResponse::Ok().json(attachments))
}

/// Generate a pre-signed download URL for an attachment
pub async fn generate_download_url(
    pool: web::Data<PgPool>,
    s3_service: web::Data<Arc<S3Service>>,
    attachment_id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let attachment_id = attachment_id.into_inner();

    // Get attachment
    let attachment = CardAttachment::find_by_id(pool.get_ref(), attachment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // Only allow confirmed attachments
    if !attachment.is_confirmed {
        return Err(AppError::BadRequest(
            "Attachment is not confirmed yet".to_string(),
        ));
    }

    // Generate pre-signed download URL
    let download_url = s3_service.generate_download_url(&attachment.s3_key).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "download_url": download_url
    })))
}

/// Delete an attachment
pub async fn delete_attachment(
    pool: web::Data<PgPool>,
    s3_service: web::Data<Arc<S3Service>>,
    sse_manager: web::Data<Arc<SseManager>>,
    attachment_id: web::Path<Uuid>,
    user: AuthenticatedUser,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let attachment_id = attachment_id.into_inner();

    // Get attachment
    let attachment = CardAttachment::find_by_id(pool.get_ref(), attachment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // Verify user owns the attachment
    if attachment.uploaded_by != user.user_id {
        return Err(AppError::Forbidden(
            "You can only delete your own attachments".to_string(),
        ));
    }

    // Check board permissions
    let board = get_board_from_card(pool.get_ref(), attachment.card_id).await?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot delete attachments from a locked board".to_string(),
        ));
    }

    let card_id = attachment.card_id;
    let s3_key = attachment.s3_key.clone();

    // Delete from database
    let deleted = CardAttachment::delete(pool.get_ref(), attachment_id).await?;
    if !deleted {
        return Err(AppError::NotFound("Attachment not found".to_string()));
    }

    // Delete from S3 (best effort - don't fail if S3 deletion fails)
    if let Err(e) = s3_service.delete_object(&s3_key).await {
        log::error!("Failed to delete S3 object {}: {}", s3_key, e);
    }

    // Broadcast SSE event
    sse_manager
        .broadcast(
            board.id,
            SseEvent::AttachmentDeleted {
                attachment_id,
                card_id,
            },
        )
        .await;

    Ok(HttpResponse::NoContent().finish())
}
