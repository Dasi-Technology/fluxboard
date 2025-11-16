use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::serde_helpers::deserialize_null_default;

/// Card model representing a card in a column
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Card {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input data for creating a new card
#[derive(Debug, Deserialize)]
pub struct CreateCardInput {
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
}

/// Input data for updating a card
#[derive(Debug, Deserialize)]
pub struct UpdateCardInput {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub description: Option<Option<String>>,
    pub position: Option<i32>,
    pub column_id: Option<Uuid>,
}

impl Card {
    /// Create a new card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Card creation data
    ///
    /// # Returns
    /// * `Result<Card, sqlx::Error>` - Created card or error
    pub async fn create(pool: &PgPool, input: CreateCardInput) -> Result<Self, sqlx::Error> {
        let card = sqlx::query_as!(
            Card,
            r#"
            INSERT INTO cards (column_id, title, description, position)
            VALUES ($1, $2, $3, $4)
            RETURNING id, column_id, title, description, position, created_at, updated_at
            "#,
            input.column_id,
            input.title,
            input.description,
            input.position
        )
        .fetch_one(pool)
        .await?;

        Ok(card)
    }

    /// Find a card by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Card UUID
    ///
    /// # Returns
    /// * `Result<Option<Card>, sqlx::Error>` - Found card or None
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let card = sqlx::query_as!(
            Card,
            r#"
            SELECT id, column_id, title, description, position, created_at, updated_at
            FROM cards
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(card)
    }

    /// Find all cards for a column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `column_id` - Column UUID
    ///
    /// # Returns
    /// * `Result<Vec<Card>, sqlx::Error>` - List of cards ordered by position
    pub async fn find_by_column_id(
        pool: &PgPool,
        column_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let cards = sqlx::query_as!(
            Card,
            r#"
            SELECT id, column_id, title, description, position, created_at, updated_at
            FROM cards
            WHERE column_id = $1
            ORDER BY position ASC
            "#,
            column_id
        )
        .fetch_all(pool)
        .await?;

        Ok(cards)
    }

    /// Find all cards for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `Result<Vec<Card>, sqlx::Error>` - List of all cards in the board
    pub async fn find_by_board_id(pool: &PgPool, board_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let cards = sqlx::query_as!(
            Card,
            r#"
            SELECT c.id, c.column_id, c.title, c.description, c.position, c.created_at, c.updated_at
            FROM cards c
            INNER JOIN columns col ON c.column_id = col.id
            WHERE col.board_id = $1
            ORDER BY col.position ASC, c.position ASC
            "#,
            board_id
        )
        .fetch_all(pool)
        .await?;

        Ok(cards)
    }

    /// Update a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Card UUID
    /// * `input` - Card update data
    ///
    /// # Returns
    /// * `Result<Option<Card>, sqlx::Error>` - Updated card or None if not found
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateCardInput,
    ) -> Result<Option<Self>, sqlx::Error> {
        // Flatten the Option<Option<String>> for description
        // None = don't update, Some(None) = set to NULL, Some(Some(v)) = set to v
        let update_description = input.description.is_some();
        let description_value = input.description.clone().flatten();

        let card = sqlx::query_as!(
            Card,
            r#"
            UPDATE cards
            SET
                title = COALESCE($2, title),
                description = CASE WHEN $6 THEN $3 ELSE description END,
                position = COALESCE($4, position),
                column_id = COALESCE($5, column_id),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, column_id, title, description, position, created_at, updated_at
            "#,
            id,
            input.title,
            description_value,
            input.position,
            input.column_id,
            update_description
        )
        .fetch_optional(pool)
        .await?;

        Ok(card)
    }

    /// Delete a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Card UUID
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if deleted, false if not found
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM cards
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Move a card to a different column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Card UUID
    /// * `new_column_id` - New column UUID
    /// * `new_position` - New position in the column
    ///
    /// # Returns
    /// * `Result<Option<Card>, sqlx::Error>` - Updated card or None if not found
    pub async fn move_to_column(
        pool: &PgPool,
        id: Uuid,
        new_column_id: Uuid,
        new_position: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        let card = sqlx::query_as!(
            Card,
            r#"
            UPDATE cards
            SET 
                column_id = $2,
                position = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, column_id, title, description, position, created_at, updated_at
            "#,
            id,
            new_column_id,
            new_position
        )
        .fetch_optional(pool)
        .await?;

        Ok(card)
    }

    /// Reorder cards within a column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `column_id` - Column UUID
    /// * `card_positions` - Vec of (card_id, new_position) tuples
    ///
    /// # Returns
    /// * `Result<(), sqlx::Error>` - Ok if successful
    pub async fn reorder(
        pool: &PgPool,
        column_id: Uuid,
        card_positions: Vec<(Uuid, i32)>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        for (card_id, position) in card_positions {
            sqlx::query!(
                r#"
                UPDATE cards
                SET position = $1, updated_at = NOW()
                WHERE id = $2 AND column_id = $3
                "#,
                position,
                card_id,
                column_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
