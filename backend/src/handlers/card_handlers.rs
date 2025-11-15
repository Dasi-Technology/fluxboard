use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::UpdateCardInput;
use crate::services::CardService;

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

/// Create a new card
pub async fn create_card(
    pool: web::Data<PgPool>,
    column_id: web::Path<Uuid>,
    input: web::Json<CreateCardRequest>,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let card = CardService::create_card(
        pool.get_ref(),
        column_id.into_inner(),
        input.title,
        input.description,
        input.position,
    )
    .await?;
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
    id: web::Path<Uuid>,
    input: web::Json<UpdateCardInput>,
) -> AppResult<HttpResponse> {
    let card =
        CardService::update_card(pool.get_ref(), id.into_inner(), input.into_inner()).await?;
    Ok(HttpResponse::Ok().json(card))
}

/// Delete a card
pub async fn delete_card(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    CardService::delete_card(pool.get_ref(), id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Move a card to a different column
pub async fn move_card(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    input: web::Json<MoveCardRequest>,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let card = CardService::move_card(
        pool.get_ref(),
        id.into_inner(),
        input.column_id,
        input.position,
    )
    .await?;
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
