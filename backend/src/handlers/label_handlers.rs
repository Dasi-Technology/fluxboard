use actix_web::{HttpRequest, HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Board, Card, UpdateBoardLabelInput};
use crate::services::BoardLabelService;
use crate::sse::events::SseEvent;
use crate::sse::manager::SseManager;

/// Helper function to check if a board operation is allowed
///
/// For locked boards, only requests with the correct password in X-Board-Password header are allowed
fn is_board_operation_allowed(board: &Board, req: &HttpRequest) -> bool {
    // If board is not locked, allow all operations
    if !board.is_locked {
        return true;
    }

    // Board is locked - check if request has correct password
    if let Some(password_header) = req.headers().get("X-Board-Password") {
        if let Ok(password_str) = password_header.to_str() {
            return password_str == board.password;
        }
    }

    // No password or wrong password - deny operation
    false
}

/// Request body for creating a board label
#[derive(Deserialize)]
pub struct CreateBoardLabelRequest {
    pub name: String,
    pub color: String,
}

/// Request body for updating a board label
#[derive(Deserialize)]
pub struct UpdateBoardLabelRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}

// ============================================================================
// Board Label Management Endpoints
// ============================================================================

/// GET /boards/:boardId/labels - List all labels for a board
pub async fn list_board_labels(
    pool: web::Data<PgPool>,
    board_id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let labels =
        BoardLabelService::get_labels_by_board_id(pool.get_ref(), board_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(labels))
}

/// POST /boards/:boardId/labels - Create a new label for a board
pub async fn create_board_label(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    board_id: web::Path<Uuid>,
    input: web::Json<CreateBoardLabelRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let b_id = board_id.into_inner();

    // Verify board exists and check if locked
    let board = Board::find_by_id(pool.get_ref(), b_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Board with ID {} not found", b_id))
        })?;

    // Check if board operation is allowed (locked boards require password)
    if !is_board_operation_allowed(&board, &req) {
        return Err(crate::error::AppError::Unauthorized(
            "Cannot create labels on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let label =
        BoardLabelService::create_label(pool.get_ref(), b_id, input.name, input.color).await?;

    // Broadcast label creation via SSE
    sse_manager
        .broadcast(
            b_id,
            SseEvent::BoardLabelCreated {
                label: label.clone(),
            },
        )
        .await;

    Ok(HttpResponse::Created().json(label))
}

/// PUT /boards/labels/:labelId - Update a board label
pub async fn update_board_label(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    label_id: web::Path<Uuid>,
    input: web::Json<UpdateBoardLabelRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let l_id = label_id.into_inner();

    // Get the label to find the board_id for broadcasting
    let existing_label = BoardLabelService::get_label_by_id(pool.get_ref(), l_id).await?;

    // Check if board is locked
    let board = Board::find_by_id(pool.get_ref(), existing_label.board_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!(
                "Board with ID {} not found",
                existing_label.board_id
            ))
        })?;

    // Check if board operation is allowed (locked boards require password)
    if !is_board_operation_allowed(&board, &req) {
        return Err(crate::error::AppError::Unauthorized(
            "Cannot update labels on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let update_input = UpdateBoardLabelInput {
        name: input.name,
        color: input.color,
    };

    let label = BoardLabelService::update_label(pool.get_ref(), l_id, update_input).await?;

    // Broadcast label update via SSE
    sse_manager
        .broadcast(
            existing_label.board_id,
            SseEvent::BoardLabelUpdated {
                label: label.clone(),
            },
        )
        .await;

    Ok(HttpResponse::Ok().json(label))
}

/// DELETE /boards/labels/:labelId - Delete a board label
pub async fn delete_board_label(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    label_id: web::Path<Uuid>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let l_id = label_id.into_inner();

    // Get the label to find the board_id for broadcasting
    let label = BoardLabelService::get_label_by_id(pool.get_ref(), l_id).await?;
    let board_id = label.board_id;

    // Check if board is locked
    let board = Board::find_by_id(pool.get_ref(), board_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Board with ID {} not found", board_id))
        })?;

    // Check if board operation is allowed (locked boards require password)
    if !is_board_operation_allowed(&board, &req) {
        return Err(crate::error::AppError::Unauthorized(
            "Cannot delete labels on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    BoardLabelService::delete_label(pool.get_ref(), l_id).await?;

    // Broadcast label deletion via SSE
    sse_manager
        .broadcast(board_id, SseEvent::BoardLabelDeleted { label_id: l_id })
        .await;

    Ok(HttpResponse::NoContent().finish())
}

// ============================================================================
// Card Label Assignment Endpoints
// ============================================================================

/// POST /cards/:cardId/labels/:labelId - Assign a label to a card
pub async fn assign_label_to_card(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let (card_id, label_id) = path.into_inner();

    // Verify card exists and get its column to find board_id
    let card = Card::find_by_id(pool.get_ref(), card_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Card not found".into()))?;
    let column = crate::models::Column::find_by_id(pool.get_ref(), card.column_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Column not found".into()))?;

    // Verify label belongs to the same board
    let label = BoardLabelService::get_label_by_id(pool.get_ref(), label_id).await?;
    if label.board_id != column.board_id {
        return Err(crate::error::AppError::NotFound(
            "Label does not belong to this board".into(),
        ));
    }

    // Check if board is locked
    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Board with ID {} not found", column.board_id))
        })?;

    // Check if board operation is allowed (locked boards require password)
    if !is_board_operation_allowed(&board, &req) {
        return Err(crate::error::AppError::Unauthorized(
            "Cannot assign labels on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    BoardLabelService::assign_label_to_card(pool.get_ref(), card_id, label_id).await?;

    // Broadcast label assignment via SSE
    sse_manager
        .broadcast(
            column.board_id,
            SseEvent::CardLabelAssigned { card_id, label },
        )
        .await;

    Ok(HttpResponse::Created().finish())
}

/// DELETE /cards/:cardId/labels/:labelId - Unassign a label from a card
pub async fn unassign_label_from_card(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let (card_id, label_id) = path.into_inner();

    // Get card and column to find board_id for broadcasting
    let card = Card::find_by_id(pool.get_ref(), card_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Card not found".into()))?;
    let column = crate::models::Column::find_by_id(pool.get_ref(), card.column_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Column not found".into()))?;

    // Check if board is locked
    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Board with ID {} not found", column.board_id))
        })?;

    // Check if board operation is allowed (locked boards require password)
    if !is_board_operation_allowed(&board, &req) {
        return Err(crate::error::AppError::Unauthorized(
            "Cannot unassign labels on a locked board. Only the board owner can edit locked boards.".to_string(),
        ));
    }

    BoardLabelService::unassign_label_from_card(pool.get_ref(), card_id, label_id).await?;

    // Broadcast label unassignment via SSE
    sse_manager
        .broadcast(
            column.board_id,
            SseEvent::CardLabelUnassigned { card_id, label_id },
        )
        .await;

    Ok(HttpResponse::NoContent().finish())
}
