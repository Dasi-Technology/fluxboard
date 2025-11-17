use actix_web::{HttpRequest, HttpResponse, web};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Board, CreateBoardInput, SetLockStateInput, UpdateBoardInput};
use crate::services::BoardService;
use crate::sse::events::SseEvent;
use crate::sse::manager::SseManager;

/// Helper function to check if a board operation is allowed
///
/// For locked boards, only requests with the correct password in X-Board-Password header are allowed
fn check_board_password(is_locked: bool, password: &str, req: &HttpRequest) -> bool {
    // If board is not locked, allow all operations
    if !is_locked {
        return true;
    }

    // Board is locked - check if request has correct password
    if let Some(password_header) = req.headers().get("X-Board-Password") {
        if let Ok(password_str) = password_header.to_str() {
            return password_str == password;
        }
    }

    // No password or wrong password - deny operation
    false
}

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
    sse_manager: web::Data<Arc<SseManager>>,
    token: web::Path<String>,
    input: web::Json<UpdateBoardInput>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let share_token = token.into_inner();

    // Get board first to check lock status
    let existing_board = BoardService::get_board_by_share_token(pool.get_ref(), &share_token).await?;

    if !check_board_password(existing_board.is_locked, &existing_board.password, &req) {
        return Err(AppError::Unauthorized(
            "Cannot update a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let board =
        BoardService::update_board_by_share_token(pool.get_ref(), &share_token, input.into_inner())
            .await?;

    // Broadcast board update via SSE
    sse_manager
        .broadcast(
            board.id,
            SseEvent::BoardUpdated {
                board: board.clone(),
            },
        )
        .await;

    Ok(HttpResponse::Ok().json(board))
}

/// Update a board
pub async fn update_board(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    id: web::Path<Uuid>,
    input: web::Json<UpdateBoardInput>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let board_id = id.into_inner();

    // Get board first to check lock status
    let existing_board = BoardService::get_board_by_id(pool.get_ref(), board_id).await?;

    if !check_board_password(existing_board.is_locked, &existing_board.password, &req) {
        return Err(AppError::Unauthorized(
            "Cannot update a locked board. Only the board owner can edit locked boards."
                .to_string(),
        ));
    }

    let board = BoardService::update_board(pool.get_ref(), board_id, input.into_inner()).await?;

    // Broadcast board update via SSE
    sse_manager
        .broadcast(
            board_id,
            SseEvent::BoardUpdated {
                board: board.clone(),
            },
        )
        .await;

    Ok(HttpResponse::Ok().json(board))
}

/// Delete a board
pub async fn delete_board(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    BoardService::delete_board(pool.get_ref(), id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Lock or unlock a board
pub async fn set_board_lock_state(
    pool: web::Data<PgPool>,
    sse_manager: web::Data<Arc<SseManager>>,
    token: web::Path<String>,
    input: web::Json<SetLockStateInput>,
) -> AppResult<HttpResponse> {
    let share_token = token.into_inner();
    let lock_input = input.into_inner();

    let board = BoardService::set_board_lock_state(
        pool.get_ref(),
        &share_token,
        &lock_input.password,
        lock_input.is_locked,
    )
    .await?;

    // Broadcast lock state change via SSE
    sse_manager
        .broadcast(
            board.id,
            SseEvent::BoardUpdated {
                board: board.clone(),
            },
        )
        .await;

    Ok(HttpResponse::Ok().json(board))
}
