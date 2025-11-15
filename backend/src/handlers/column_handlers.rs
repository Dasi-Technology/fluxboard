use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::UpdateColumnInput;
use crate::services::ColumnService;
use crate::sse::events::SseEvent;
use crate::sse::manager::SseManager;

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
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let b_id = board_id.into_inner();
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
) -> AppResult<HttpResponse> {
    let column =
        ColumnService::update_column(pool.get_ref(), id.into_inner(), input.into_inner()).await?;

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
) -> AppResult<HttpResponse> {
    let column_id = id.into_inner();

    // Get column before deletion to get board_id
    if let Ok(Some(column)) = crate::models::Column::find_by_id(pool.get_ref(), column_id).await {
        ColumnService::delete_column(pool.get_ref(), column_id).await?;

        // Broadcast column deletion via SSE
        sse_manager
            .broadcast(column.board_id, SseEvent::ColumnDeleted { column_id })
            .await;
    } else {
        ColumnService::delete_column(pool.get_ref(), column_id).await?;
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Reorder columns within a board
pub async fn reorder_columns(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    board_id: web::Path<Uuid>,
    input: web::Json<ReorderColumnsRequest>,
) -> AppResult<HttpResponse> {
    let b_id = board_id.into_inner();
    let column_positions = input.into_inner().column_positions;

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
