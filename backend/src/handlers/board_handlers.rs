use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{CreateBoardInput, UpdateBoardInput};
use crate::services::BoardService;

/// Create a new board
pub async fn create_board(
    pool: web::Data<PgPool>,
    input: web::Json<CreateBoardInput>,
) -> AppResult<HttpResponse> {
    let board = BoardService::create_board(pool.get_ref(), input.into_inner()).await?;
    Ok(HttpResponse::Created().json(board))
}

/// List all boards
pub async fn list_boards(pool: web::Data<PgPool>) -> AppResult<HttpResponse> {
    let boards = BoardService::list_boards(pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(boards))
}

/// Get a board by ID
pub async fn get_board(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let board = BoardService::get_board_by_id(pool.get_ref(), id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(board))
}

/// Get a board by share token
pub async fn get_board_by_share_token(
    pool: web::Data<PgPool>,
    token: web::Path<String>,
) -> AppResult<HttpResponse> {
    let board = BoardService::get_board_by_share_token(pool.get_ref(), &token.into_inner()).await?;
    Ok(HttpResponse::Ok().json(board))
}

/// Update a board by share token
pub async fn update_board_by_share_token(
    pool: web::Data<PgPool>,
    token: web::Path<String>,
    input: web::Json<UpdateBoardInput>,
) -> AppResult<HttpResponse> {
    let board = BoardService::update_board_by_share_token(
        pool.get_ref(),
        &token.into_inner(),
        input.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(board))
}

/// Update a board
pub async fn update_board(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateBoardInput>,
) -> AppResult<HttpResponse> {
    let board =
        BoardService::update_board(pool.get_ref(), id.into_inner(), input.into_inner()).await?;
    Ok(HttpResponse::Ok().json(board))
}

/// Delete a board
pub async fn delete_board(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    BoardService::delete_board(pool.get_ref(), id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
