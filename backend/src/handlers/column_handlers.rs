use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::UpdateColumnInput;
use crate::services::ColumnService;

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
    board_id: web::Path<Uuid>,
    input: web::Json<CreateColumnRequest>,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let column = ColumnService::create_column(
        pool.get_ref(),
        board_id.into_inner(),
        input.title,
        input.position,
    )
    .await?;
    Ok(HttpResponse::Created().json(column))
}

/// Update a column
pub async fn update_column(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateColumnInput>,
) -> AppResult<HttpResponse> {
    let column =
        ColumnService::update_column(pool.get_ref(), id.into_inner(), input.into_inner()).await?;
    Ok(HttpResponse::Ok().json(column))
}

/// Delete a column
pub async fn delete_column(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ColumnService::delete_column(pool.get_ref(), id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Reorder columns within a board
pub async fn reorder_columns(
    pool: web::Data<PgPool>,
    board_id: web::Path<Uuid>,
    input: web::Json<ReorderColumnsRequest>,
) -> AppResult<HttpResponse> {
    ColumnService::reorder_columns(
        pool.get_ref(),
        board_id.into_inner(),
        input.into_inner().column_positions,
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}
