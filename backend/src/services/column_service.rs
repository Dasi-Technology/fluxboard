use crate::error::{AppError, AppResult};
use crate::models::{Column, CreateColumnInput, UpdateColumnInput};
use sqlx::PgPool;
use uuid::Uuid;

/// Service for column-related business logic
pub struct ColumnService;

impl ColumnService {
    /// Create a new column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    /// * `title` - Column title
    /// * `position` - Column position
    ///
    /// # Returns
    /// * `AppResult<Column>` - Created column or error
    pub async fn create_column(
        pool: &PgPool,
        board_id: Uuid,
        title: String,
        position: i32,
    ) -> AppResult<Column> {
        // Validate input
        if title.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Column title cannot be empty".to_string(),
            ));
        }

        if title.len() > 255 {
            return Err(AppError::BadRequest(
                "Column title cannot exceed 255 characters".to_string(),
            ));
        }

        if position < 0 {
            return Err(AppError::BadRequest(
                "Column position cannot be negative".to_string(),
            ));
        }

        let input = CreateColumnInput {
            board_id,
            title,
            position,
        };

        let column = Column::create(pool, input).await?;
        Ok(column)
    }

    /// Get column by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Column UUID
    ///
    /// # Returns
    /// * `AppResult<Column>` - Found column or error
    pub async fn get_column_by_id(pool: &PgPool, id: Uuid) -> AppResult<Column> {
        Column::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Column with ID {} not found", id)))
    }

    /// Get all columns for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `AppResult<Vec<Column>>` - List of columns ordered by position
    pub async fn get_columns_by_board_id(pool: &PgPool, board_id: Uuid) -> AppResult<Vec<Column>> {
        let columns = Column::find_by_board_id(pool, board_id).await?;
        Ok(columns)
    }

    /// Update a column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Column UUID
    /// * `input` - Column update data
    ///
    /// # Returns
    /// * `AppResult<Column>` - Updated column or error
    pub async fn update_column(
        pool: &PgPool,
        id: Uuid,
        input: UpdateColumnInput,
    ) -> AppResult<Column> {
        // Validate title if provided
        if let Some(ref title) = input.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Column title cannot be empty".to_string(),
                ));
            }
            if title.len() > 255 {
                return Err(AppError::BadRequest(
                    "Column title cannot exceed 255 characters".to_string(),
                ));
            }
        }

        // Validate position if provided
        if let Some(position) = input.position {
            if position < 0 {
                return Err(AppError::BadRequest(
                    "Column position cannot be negative".to_string(),
                ));
            }
        }

        Column::update(pool, id, input)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Column with ID {} not found", id)))
    }

    /// Delete a column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Column UUID
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn delete_column(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let deleted = Column::delete(pool, id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "Column with ID {} not found",
                id
            )))
        }
    }

    /// Reorder columns within a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    /// * `column_positions` - Vec of (column_id, new_position) tuples
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn reorder_columns(
        pool: &PgPool,
        board_id: Uuid,
        column_positions: Vec<(Uuid, i32)>,
    ) -> AppResult<()> {
        // Validate positions
        for (_, position) in &column_positions {
            if *position < 0 {
                return Err(AppError::BadRequest(
                    "Column position cannot be negative".to_string(),
                ));
            }
        }

        Column::reorder(pool, board_id, column_positions).await?;
        Ok(())
    }
}
