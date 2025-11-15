use actix::Addr;
use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Column, UpdateCardInput};
use crate::services::CardService;
use crate::websocket::messages::{WsMessage, WsMessageType};
use crate::websocket::server::{Broadcast, WsServer};

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
    ws_server: web::Data<Addr<WsServer>>,
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
        // Broadcast card creation via WebSocket
        log::info!(
            "Broadcasting card creation: {} to board: {}",
            card.id,
            column.board_id
        );
        let ws_message = WsMessage::new(column.board_id, WsMessageType::CardCreated(card.clone()));
        ws_server.do_send(Broadcast {
            message: ws_message,
        });
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
    ws_server: web::Data<Addr<WsServer>>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateCardInput>,
) -> AppResult<HttpResponse> {
    let card_id = id.into_inner();
    let card = CardService::update_card(pool.get_ref(), card_id, input.into_inner()).await?;

    // Get the column to find the board_id
    if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
        // Broadcast card update via WebSocket
        log::info!(
            "Broadcasting card update: {} to board: {}",
            card.id,
            column.board_id
        );
        let ws_message = WsMessage::new(column.board_id, WsMessageType::CardUpdated(card.clone()));
        ws_server.do_send(Broadcast {
            message: ws_message,
        });
    }

    Ok(HttpResponse::Ok().json(card))
}

/// Delete a card
pub async fn delete_card(
    pool: web::Data<PgPool>,
    ws_server: web::Data<Addr<WsServer>>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let card_id = id.into_inner();

    // Get card and column before deletion to broadcast
    if let Ok(Some(card)) = crate::models::Card::find_by_id(pool.get_ref(), card_id).await {
        if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
            CardService::delete_card(pool.get_ref(), card_id).await?;

            // Broadcast card deletion via WebSocket
            log::info!(
                "Broadcasting card deletion: {} from board: {}",
                card_id,
                column.board_id
            );
            let ws_message =
                WsMessage::new(column.board_id, WsMessageType::CardDeleted { id: card_id });
            ws_server.do_send(Broadcast {
                message: ws_message,
            });
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Move a card to a different column
pub async fn move_card(
    pool: web::Data<PgPool>,
    ws_server: web::Data<Addr<WsServer>>,
    id: web::Path<Uuid>,
    input: web::Json<MoveCardRequest>,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let card_id = id.into_inner();
    let card =
        CardService::move_card(pool.get_ref(), card_id, input.column_id, input.position).await?;

    // Get the column to find the board_id
    if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
        // Broadcast card moved via WebSocket
        log::info!(
            "Broadcasting card move: {} to board: {}",
            card.id,
            column.board_id
        );
        let ws_message = WsMessage::new(
            column.board_id,
            WsMessageType::CardMoved {
                id: card.id,
                column_id: card.column_id,
                position: card.position,
            },
        );
        ws_server.do_send(Broadcast {
            message: ws_message,
        });
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
