use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Column, UpdateCardInput};
use crate::services::{AiService, CardService};
use crate::sse::events::SseEvent;
use crate::sse::manager::SseManager;

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
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let col_id = column_id.into_inner();
    let card = CardService::create_card(
        pool.get_ref(),
        col_id,
        input.title,
        input.description,
        input.position,
    )
    .await?;

    // Get the column to find the board_id
    if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), col_id).await {
        // Broadcast card creation via SSE
        sse_manager
            .broadcast(
                column.board_id,
                SseEvent::CardCreated { card: card.clone() },
            )
            .await;
    }

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
) -> AppResult<HttpResponse> {
    let card_id = id.into_inner();
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
) -> AppResult<HttpResponse> {
    let card_id = id.into_inner();

    // Get card and column before deletion to broadcast
    if let Ok(Some(card)) = crate::models::Card::find_by_id(pool.get_ref(), card_id).await {
        if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
            CardService::delete_card(pool.get_ref(), card_id).await?;

            // Broadcast card deletion via SSE
            sse_manager
                .broadcast(column.board_id, SseEvent::CardDeleted { card_id })
                .await;
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Move a card to a different column
pub async fn move_card(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    input: web::Json<MoveCardRequest>,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let card_id = id.into_inner();

    // Get the card before moving to know the from_column_id
    let from_column_id =
        if let Ok(Some(card)) = crate::models::Card::find_by_id(pool.get_ref(), card_id).await {
            card.column_id
        } else {
            input.column_id // fallback, though this shouldn't happen
        };

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
    column_id: web::Path<Uuid>,
    input: web::Json<ReorderCardsRequest>,
) -> AppResult<HttpResponse> {
    CardService::reorder_cards(
        pool.get_ref(),
        column_id.into_inner(),
        input.into_inner().card_positions,
    )
    .await?;
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
