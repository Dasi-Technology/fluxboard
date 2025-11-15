use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Column model representing a column in a Kanban board
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Column {
    pub id: Uuid,
    pub board_id: Uuid,
    pub title: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input data for creating a new column
#[derive(Debug, Deserialize)]
pub struct CreateColumnInput {
    pub board_id: Uuid,
    pub title: String,
    pub position: i32,
}

/// Input data for updating a column
#[derive(Debug, Deserialize)]
pub struct UpdateColumnInput {
    pub title: Option<String>,
    pub position: Option<i32>,
}

impl Column {
    /// Create a new column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Column creation data
    ///
    /// # Returns
    /// * `Result<Column, sqlx::Error>` - Created column or error
    pub async fn create(pool: &PgPool, input: CreateColumnInput) -> Result<Self, sqlx::Error> {
        let column = sqlx::query_as!(
            Column,
            r#"
            INSERT INTO columns (board_id, title, position)
            VALUES ($1, $2, $3)
            RETURNING id, board_id, title, position, created_at, updated_at
            "#,
            input.board_id,
            input.title,
            input.position
        )
        .fetch_one(pool)
        .await?;

        Ok(column)
    }

    /// Find a column by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Column UUID
    ///
    /// # Returns
    /// * `Result<Option<Column>, sqlx::Error>` - Found column or None
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let column = sqlx::query_as!(
            Column,
            r#"
            SELECT id, board_id, title, position, created_at, updated_at
            FROM columns
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(column)
    }

    /// Find all columns for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `Result<Vec<Column>, sqlx::Error>` - List of columns ordered by position
    pub async fn find_by_board_id(pool: &PgPool, board_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let columns = sqlx::query_as!(
            Column,
            r#"
            SELECT id, board_id, title, position, created_at, updated_at
            FROM columns
            WHERE board_id = $1
            ORDER BY position ASC
            "#,
            board_id
        )
        .fetch_all(pool)
        .await?;

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
    /// * `Result<Option<Column>, sqlx::Error>` - Updated column or None if not found
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateColumnInput,
    ) -> Result<Option<Self>, sqlx::Error> {
        let column = sqlx::query_as!(
            Column,
            r#"
            UPDATE columns
            SET 
                title = COALESCE($2, title),
                position = COALESCE($3, position),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, board_id, title, position, created_at, updated_at
            "#,
            id,
            input.title,
            input.position
        )
        .fetch_optional(pool)
        .await?;

        Ok(column)
    }

    /// Delete a column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Column UUID
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if deleted, false if not found
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM columns
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Reorder columns for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    /// * `column_positions` - Vec of (column_id, new_position) tuples
    ///
    /// # Returns
    /// * `Result<(), sqlx::Error>` - Ok if successful
    pub async fn reorder(
        pool: &PgPool,
        board_id: Uuid,
        column_positions: Vec<(Uuid, i32)>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        for (column_id, position) in column_positions {
            sqlx::query!(
                r#"
                UPDATE columns
                SET position = $1, updated_at = NOW()
                WHERE id = $2 AND board_id = $3
                "#,
                position,
                column_id,
                board_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
