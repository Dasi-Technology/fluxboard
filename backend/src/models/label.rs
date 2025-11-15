use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Label model representing a label attached to a card
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Label {
    pub id: Uuid,
    pub card_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

/// Input data for creating a new label
#[derive(Debug, Deserialize)]
pub struct CreateLabelInput {
    pub card_id: Uuid,
    pub name: String,
    pub color: String,
}

/// Input data for updating a label
#[derive(Debug, Deserialize)]
pub struct UpdateLabelInput {
    pub name: Option<String>,
    pub color: Option<String>,
}

impl Label {
    /// Create a new label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Label creation data
    ///
    /// # Returns
    /// * `Result<Label, sqlx::Error>` - Created label or error
    pub async fn create(pool: &PgPool, input: CreateLabelInput) -> Result<Self, sqlx::Error> {
        let label = sqlx::query_as!(
            Label,
            r#"
            INSERT INTO labels (card_id, name, color)
            VALUES ($1, $2, $3)
            RETURNING id, card_id, name, color, created_at
            "#,
            input.card_id,
            input.name,
            input.color
        )
        .fetch_one(pool)
        .await?;

        Ok(label)
    }

    /// Find a label by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    ///
    /// # Returns
    /// * `Result<Option<Label>, sqlx::Error>` - Found label or None
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let label = sqlx::query_as!(
            Label,
            r#"
            SELECT id, card_id, name, color, created_at
            FROM labels
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(label)
    }

    /// Find all labels for a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `Result<Vec<Label>, sqlx::Error>` - List of labels for the card
    pub async fn find_by_card_id(pool: &PgPool, card_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let labels = sqlx::query_as!(
            Label,
            r#"
            SELECT id, card_id, name, color, created_at
            FROM labels
            WHERE card_id = $1
            ORDER BY created_at ASC
            "#,
            card_id
        )
        .fetch_all(pool)
        .await?;

        Ok(labels)
    }

    /// Find all labels for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `Result<Vec<Label>, sqlx::Error>` - List of all labels in the board
    pub async fn find_by_board_id(pool: &PgPool, board_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let labels = sqlx::query_as!(
            Label,
            r#"
            SELECT l.id, l.card_id, l.name, l.color, l.created_at
            FROM labels l
            INNER JOIN cards c ON l.card_id = c.id
            INNER JOIN columns col ON c.column_id = col.id
            WHERE col.board_id = $1
            ORDER BY l.created_at ASC
            "#,
            board_id
        )
        .fetch_all(pool)
        .await?;

        Ok(labels)
    }

    /// Update a label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    /// * `input` - Label update data
    ///
    /// # Returns
    /// * `Result<Option<Label>, sqlx::Error>` - Updated label or None if not found
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateLabelInput,
    ) -> Result<Option<Self>, sqlx::Error> {
        let label = sqlx::query_as!(
            Label,
            r#"
            UPDATE labels
            SET 
                name = COALESCE($2, name),
                color = COALESCE($3, color)
            WHERE id = $1
            RETURNING id, card_id, name, color, created_at
            "#,
            id,
            input.name,
            input.color
        )
        .fetch_optional(pool)
        .await?;

        Ok(label)
    }

    /// Delete a label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if deleted, false if not found
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM labels
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all labels for a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `Result<u64, sqlx::Error>` - Number of labels deleted
    pub async fn delete_by_card_id(pool: &PgPool, card_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM labels
            WHERE card_id = $1
            "#,
            card_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
