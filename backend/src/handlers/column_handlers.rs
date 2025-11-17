use actix_web::{HttpRequest, HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Board, UpdateColumnInput};
use crate::services::ColumnService;
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

/// Request body for creating a column
#[derive(Deserialize)]
pub struct CreateColumnRequest {
    pub title: String,
    pub position: i32,
}

/// Request body for reordering columns
#[derive(Deserialize)]
pub struct ReorderColumnsRequest {
    pub column_positions: Vec<(Uuid, i32)>,
}

/// Create a new column
pub async fn create_column(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    board_id: web::Path<Uuid>,
    input: web::Json<CreateColumnRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let b_id = board_id.into_inner();

    // Check board lock status
    let board = Board::find_by_id(pool.get_ref(), b_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot create columns on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let column =
        ColumnService::create_column(pool.get_ref(), b_id, input.title, input.position).await?;

    // Broadcast column creation via SSE
    sse_manager
        .broadcast(
            b_id,
            SseEvent::ColumnCreated {
                column: column.clone(),
            },
        )
        .await;

    Ok(HttpResponse::Created().json(column))
}

/// Update a column
pub async fn update_column(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateColumnInput>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let column_id = id.into_inner();

    // Get column to find board_id, then check board lock status
    let existing_column = crate::models::Column::find_by_id(pool.get_ref(), column_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool.get_ref(), existing_column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot update columns on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let column = ColumnService::update_column(pool.get_ref(), column_id, input.into_inner()).await?;

    // Broadcast column update via SSE
    sse_manager
        .broadcast(
            column.board_id,
            SseEvent::ColumnUpdated {
                column: column.clone(),
            },
        )
        .await;

    Ok(HttpResponse::Ok().json(column))
}

/// Delete a column
pub async fn delete_column(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let column_id = id.into_inner();

    // Get column before deletion to get board_id and check lock status
    let column = crate::models::Column::find_by_id(pool.get_ref(), column_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot delete columns on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    ColumnService::delete_column(pool.get_ref(), column_id).await?;

    // Broadcast column deletion via SSE
    sse_manager
        .broadcast(column.board_id, SseEvent::ColumnDeleted { column_id })
        .await;

    Ok(HttpResponse::NoContent().finish())
}

/// Reorder columns within a board
pub async fn reorder_columns(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    board_id: web::Path<Uuid>,
    input: web::Json<ReorderColumnsRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let b_id = board_id.into_inner();
    let column_positions = input.into_inner().column_positions;

    // Check board lock status
    let board = Board::find_by_id(pool.get_ref(), b_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot reorder columns on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    ColumnService::reorder_columns(pool.get_ref(), b_id, column_positions.clone()).await?;

    // Broadcast column reordering via SSE for each column
    for (column_id, new_position) in column_positions {
        sse_manager
            .broadcast(
                b_id,
                SseEvent::ColumnReordered {
                    column_id,
                    new_position,
                },
            )
            .await;
    }

    Ok(HttpResponse::Ok().finish())
}
