use crate::error::{AppError, AppResult};
use crate::models::{Board, BoardWithRelations, CreateBoardInput, UpdateBoardInput};
use sqlx::PgPool;
use uuid::Uuid;

/// Service for board-related business logic
pub struct BoardService;

impl BoardService {
    /// Create a new board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Board creation data
    ///
    /// # Returns
    /// * `AppResult<Board>` - Created board or error
    pub async fn create_board(pool: &PgPool, input: CreateBoardInput) -> AppResult<Board> {
        // Validate input
        if input.title.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Board title cannot be empty".to_string(),
            ));
        }

        if input.title.len() > 255 {
            return Err(AppError::BadRequest(
                "Board title cannot exceed 255 characters".to_string(),
            ));
        }

        // Create board using model
        let board = Board::create(pool, input).await?;
        Ok(board)
    }

    /// Get board by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Board UUID
    ///
    /// # Returns
    /// * `AppResult<Board>` - Found board or error
    pub async fn get_board_by_id(pool: &PgPool, id: Uuid) -> AppResult<Board> {
        Board::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Board with ID {} not found", id)))
    }

    /// Get board by share token with all relations
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `share_token` - Share token string
    ///
    /// # Returns
    /// * `AppResult<BoardWithRelations>` - Found board with relations or error
    pub async fn get_board_by_share_token(
        pool: &PgPool,
        share_token: &str,
    ) -> AppResult<BoardWithRelations> {
        Board::find_by_share_token_with_relations(pool, share_token)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Board with share token '{}' not found",
                    share_token
                ))
            })
    }

    /// Update board by share token
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `share_token` - Share token string
    /// * `input` - Board update data
    ///
    /// # Returns
    /// * `AppResult<Board>` - Updated board or error
    pub async fn update_board_by_share_token(
        pool: &PgPool,
        share_token: &str,
        input: UpdateBoardInput,
    ) -> AppResult<Board> {
        // First get the board by share token to get its ID
        let board = Self::get_board_by_share_token(pool, share_token).await?;

        // Then update using the ID
        Self::update_board(pool, board.id, input).await
    }

    /// List all boards
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `AppResult<Vec<Board>>` - List of all boards
    pub async fn list_boards(pool: &PgPool) -> AppResult<Vec<Board>> {
        let boards = Board::list_all(pool).await?;
        Ok(boards)
    }

    /// Update a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Board UUID
    /// * `input` - Board update data
    ///
    /// # Returns
    /// * `AppResult<Board>` - Updated board or error
    pub async fn update_board(
        pool: &PgPool,
        id: Uuid,
        input: UpdateBoardInput,
    ) -> AppResult<Board> {
        // Validate title if provided
        if let Some(ref title) = input.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Board title cannot be empty".to_string(),
                ));
            }
            if title.len() > 255 {
                return Err(AppError::BadRequest(
                    "Board title cannot exceed 255 characters".to_string(),
                ));
            }
        }

        Board::update(pool, id, input)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Board with ID {} not found", id)))
    }

    /// Delete a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Board UUID
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn delete_board(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let deleted = Board::delete(pool, id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "Board with ID {} not found",
                id
            )))
        }
    }

    /// Lock or unlock a board with password verification
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `share_token` - Board share token
    /// * `password` - Password to verify
    /// * `is_locked` - New lock state
    ///
    /// # Returns
    /// * `AppResult<Board>` - Updated board or error
    pub async fn set_board_lock_state(
        pool: &PgPool,
        share_token: &str,
        password: &str,
        is_locked: bool,
    ) -> AppResult<Board> {
        // First get the board by share token to get its ID
        let board = Board::find_by_share_token(pool, share_token)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Board with share token '{}' not found",
                    share_token
                ))
            })?;

        // Attempt to set lock state with password verification
        let updated_board = Board::set_lock_state(pool, board.id, password, is_locked)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid password".to_string()))?;

        Ok(updated_board)
    }
}
