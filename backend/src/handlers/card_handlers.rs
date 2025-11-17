use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Board, Column, UpdateCardInput};
use crate::services::{AiService, CardService};
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

/// Request body for creating a card
#[derive(Deserialize)]
pub struct CreateCardRequest {
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
}

/// Request body for moving a card
#[derive(Deserialize)]
pub struct MoveCardRequest {
    pub column_id: Uuid,
    pub position: i32,
}

/// Request body for reordering cards
#[derive(Deserialize)]
pub struct ReorderCardsRequest {
    pub card_positions: Vec<(Uuid, i32)>,
}

/// Request body for AI generation
#[derive(Deserialize)]
pub struct GenerateDescriptionRequest {
    pub title: String,
    pub context: Option<String>,
    pub format: DescriptionFormat,
}

/// Description format type
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DescriptionFormat {
    Bullets,
    Long,
}

/// Response for AI generation
#[derive(Serialize)]
pub struct GenerateDescriptionResponse {
    pub description: String,
}

/// Create a new card
pub async fn create_card(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    column_id: web::Path<Uuid>,
    input: web::Json<CreateCardRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let col_id = column_id.into_inner();

    // Get the column to find the board_id and check lock status
    let column = Column::find_by_id(pool.get_ref(), col_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot create cards on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let card = CardService::create_card(
        pool.get_ref(),
        col_id,
        input.title,
        input.description,
        input.position,
    )
    .await?;

    // Broadcast card creation via SSE
    sse_manager
        .broadcast(
            column.board_id,
            SseEvent::CardCreated { card: card.clone() },
        )
        .await;

    Ok(HttpResponse::Created().json(card))
}

/// Get a card by ID
pub async fn get_card(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let card = CardService::get_card_by_id(pool.get_ref(), id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(card))
}

/// Update a card
pub async fn update_card(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateCardInput>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let card_id = id.into_inner();

    // Get card to find column_id, then check board lock status
    let existing_card = crate::models::Card::find_by_id(pool.get_ref(), card_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Card not found".to_string()))?;

    let column = Column::find_by_id(pool.get_ref(), existing_card.column_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot update cards on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let card = CardService::update_card(pool.get_ref(), card_id, input.into_inner()).await?;

    // Get the column to find the board_id
    if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
        // Broadcast card update via SSE
        sse_manager
            .broadcast(
                column.board_id,
                SseEvent::CardUpdated { card: card.clone() },
            )
            .await;
    }

    Ok(HttpResponse::Ok().json(card))
}

/// Delete a card
pub async fn delete_card(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let card_id = id.into_inner();

    // Get card and column before deletion to check lock status and broadcast
    let card = crate::models::Card::find_by_id(pool.get_ref(), card_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Card not found".to_string()))?;

    let column = Column::find_by_id(pool.get_ref(), card.column_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot delete cards on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    CardService::delete_card(pool.get_ref(), card_id).await?;

    // Broadcast card deletion via SSE
    sse_manager
        .broadcast(column.board_id, SseEvent::CardDeleted { card_id })
        .await;

    Ok(HttpResponse::NoContent().finish())
}

/// Move a card to a different column
pub async fn move_card(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    input: web::Json<MoveCardRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let card_id = id.into_inner();

    // Get the card before moving to know the from_column_id and check lock status
    let card = crate::models::Card::find_by_id(pool.get_ref(), card_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Card not found".to_string()))?;

    let from_column_id = card.column_id;

    let column = Column::find_by_id(pool.get_ref(), from_column_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot move cards on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let card =
        CardService::move_card(pool.get_ref(), card_id, input.column_id, input.position).await?;

    // Get the column to find the board_id
    if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
        // Broadcast card moved via SSE
        sse_manager
            .broadcast(
                column.board_id,
                SseEvent::CardMoved {
                    card_id: card.id,
                    from_column_id,
                    to_column_id: card.column_id,
                    new_position: card.position,
                },
            )
            .await;
    }

    Ok(HttpResponse::Ok().json(card))
}

/// Reorder cards within a column
pub async fn reorder_cards(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    column_id: web::Path<Uuid>,
    input: web::Json<ReorderCardsRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let col_id = column_id.into_inner();
    let card_positions = input.into_inner().card_positions;

    // Get the board_id from the column and check lock status
    let column = crate::models::Column::find_by_id(pool.get_ref(), col_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let board = Board::find_by_id(pool.get_ref(), column.board_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if !is_board_operation_allowed(&board, &req) {
        return Err(AppError::Unauthorized(
            "Cannot reorder cards on a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    CardService::reorder_cards(pool.get_ref(), col_id, card_positions.clone()).await?;

    // Broadcast SSE events for each reordered card
    for (card_id, new_position) in card_positions {
        sse_manager
            .broadcast(
                column.board_id,
                SseEvent::CardReordered {
                    card_id,
                    column_id: col_id,
                    new_position,
                },
            )
            .await;
    }

    Ok(HttpResponse::Ok().finish())
}

/// Generate AI description for a card
pub async fn generate_description(
    ai_service: Option<web::Data<Arc<AiService>>>,
    input: web::Json<GenerateDescriptionRequest>,
) -> AppResult<HttpResponse> {
    // Check if AI service is available
    let ai_service = ai_service.ok_or_else(|| {
        AppError::BadRequest(
            "AI service not configured. Please add GEMINI_API_KEY to .env".to_string(),
        )
    })?;

    let input = input.into_inner();
    let context = input.context.unwrap_or_default();

    let description = match input.format {
        DescriptionFormat::Bullets => {
            ai_service
                .generate_bullet_points(&input.title, &context)
                .await?
        }
        DescriptionFormat::Long => {
            ai_service
                .generate_long_description(&input.title, &context)
                .await?
        }
    };

    Ok(HttpResponse::Ok().json(GenerateDescriptionResponse { description }))
}
