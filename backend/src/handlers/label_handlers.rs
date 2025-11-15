use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::UpdateLabelInput;
use crate::services::LabelService;

/// Request body for creating a label
#[derive(Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: String,
}

/// Create a new label
pub async fn create_label(
    pool: web::Data<PgPool>,
    card_id: web::Path<Uuid>,
    input: web::Json<CreateLabelRequest>,
) -> AppResult<HttpResponse> {
    let input = input.into_inner();
    let label = LabelService::create_label(
        pool.get_ref(),
        card_id.into_inner(),
        input.name,
        input.color,
    )
    .await?;
    Ok(HttpResponse::Created().json(label))
}

/// Update a label
pub async fn update_label(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateLabelInput>,
) -> AppResult<HttpResponse> {
    let label =
        LabelService::update_label(pool.get_ref(), id.into_inner(), input.into_inner()).await?;
    Ok(HttpResponse::Ok().json(label))
}

/// Delete a label
pub async fn delete_label(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    LabelService::delete_label(pool.get_ref(), id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
