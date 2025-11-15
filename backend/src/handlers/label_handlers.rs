use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Card, Column, UpdateLabelInput};
use crate::services::LabelService;
use crate::sse::events::SseEvent;
use crate::sse::manager::SseManager;

/// Request body for creating a label
#[derive(Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: String,
}

/// Create a new label
pub async fn create_label(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    card_id: web::Path<Uuid>,
    input: web::Json<CreateLabelRequest>,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let c_id = card_id.into_inner();
    let label = LabelService::create_label(pool.get_ref(), c_id, input.name, input.color).await?;

    // Get the card and column to find board_id
    if let Ok(Some(card)) = Card::find_by_id(pool.get_ref(), c_id).await {
        if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
            // Broadcast label creation via SSE
            sse_manager
                .broadcast(
                    column.board_id,
                    SseEvent::LabelCreated {
                        label: label.clone(),
                    },
                )
                .await;
        }
    }

    Ok(HttpResponse::Created().json(label))
}

/// Update a label
pub async fn update_label(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateLabelInput>,
) -> AppResult<HttpResponse> {
    let label =
        LabelService::update_label(pool.get_ref(), id.into_inner(), input.into_inner()).await?;

    // Get the card and column to find board_id
    if let Ok(Some(card)) = Card::find_by_id(pool.get_ref(), label.card_id).await {
        if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
            // Broadcast label update via SSE
            sse_manager
                .broadcast(
                    column.board_id,
                    SseEvent::LabelUpdated {
                        label: label.clone(),
                    },
                )
                .await;
        }
    }

    Ok(HttpResponse::Ok().json(label))
}

/// Delete a label
pub async fn delete_label(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let label_id = id.into_inner();

    // Get label, card, and column before deletion to broadcast
    if let Ok(Some(label)) = crate::models::Label::find_by_id(pool.get_ref(), label_id).await {
        if let Ok(Some(card)) = Card::find_by_id(pool.get_ref(), label.card_id).await {
            if let Ok(Some(column)) = Column::find_by_id(pool.get_ref(), card.column_id).await {
                LabelService::delete_label(pool.get_ref(), label_id).await?;

                // Broadcast label deletion via SSE
                sse_manager
                    .broadcast(column.board_id, SseEvent::LabelDeleted { label_id })
                    .await;

                return Ok(HttpResponse::NoContent().finish());
            }
        }
    }

    LabelService::delete_label(pool.get_ref(), label_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
