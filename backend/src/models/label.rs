use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Board-level label model (replaces card-level Label)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BoardLabel {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Card-label assignment (junction table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CardLabel {
    pub card_id: Uuid,
    pub label_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Input data for creating a new board label
#[derive(Debug, Deserialize)]
pub struct CreateBoardLabelInput {
    pub board_id: Uuid,
    pub name: String,
    pub color: String,
}

/// Input data for updating a board label
#[derive(Debug, Deserialize)]
pub struct UpdateBoardLabelInput {
    pub name: Option<String>,
    pub color: Option<String>,
}

impl BoardLabel {
    /// Create a new board label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Label creation data
    ///
    /// # Returns
    /// * `Result<BoardLabel, sqlx::Error>` - Created label or error
    pub async fn create(pool: &PgPool, input: CreateBoardLabelInput) -> Result<Self, sqlx::Error> {
        let label = sqlx::query_as!(
            BoardLabel,
            r#"
            INSERT INTO board_labels (board_id, name, color)
            VALUES ($1, $2, $3)
            RETURNING id, board_id, name, color, created_at, updated_at
            "#,
            input.board_id,
            input.name,
            input.color
        )
        .fetch_one(pool)
        .await?;

        Ok(label)
    }

    /// Find a board label by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    ///
    /// # Returns
    /// * `Result<Option<BoardLabel>, sqlx::Error>` - Found label or None
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let label = sqlx::query_as!(
            BoardLabel,
            r#"
            SELECT id, board_id, name, color, created_at, updated_at
            FROM board_labels
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(label)
    }

    /// Find all labels for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `Result<Vec<BoardLabel>, sqlx::Error>` - List of all labels for the board
    pub async fn find_by_board_id(pool: &PgPool, board_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let labels = sqlx::query_as!(
            BoardLabel,
            r#"
            SELECT id, board_id, name, color, created_at, updated_at
            FROM board_labels
            WHERE board_id = $1
            ORDER BY created_at ASC
            "#,
            board_id
        )
        .fetch_all(pool)
        .await?;

        Ok(labels)
    }

    /// Find all labels assigned to a specific card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `Result<Vec<BoardLabel>, sqlx::Error>` - List of labels assigned to the card
    pub async fn find_by_card_id(pool: &PgPool, card_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let labels = sqlx::query_as!(
            BoardLabel,
            r#"
            SELECT bl.id, bl.board_id, bl.name, bl.color, bl.created_at, bl.updated_at
            FROM board_labels bl
            INNER JOIN card_labels cl ON bl.id = cl.label_id
            WHERE cl.card_id = $1
            ORDER BY bl.created_at ASC
            "#,
            card_id
        )
        .fetch_all(pool)
        .await?;

        Ok(labels)
    }

    /// Update a board label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    /// * `input` - Label update data
    ///
    /// # Returns
    /// * `Result<Option<BoardLabel>, sqlx::Error>` - Updated label or None if not found
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateBoardLabelInput,
    ) -> Result<Option<Self>, sqlx::Error> {
        let label = sqlx::query_as!(
            BoardLabel,
            r#"
            UPDATE board_labels
            SET
                name = COALESCE($2, name),
                color = COALESCE($3, color)
            WHERE id = $1
            RETURNING id, board_id, name, color, created_at, updated_at
            "#,
            id,
            input.name,
            input.color
        )
        .fetch_optional(pool)
        .await?;

        Ok(label)
    }

    /// Delete a board label
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
            DELETE FROM board_labels
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

impl CardLabel {
    /// Assign a label to a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    /// * `label_id` - Label UUID
    ///
    /// # Returns
    /// * `Result<CardLabel, sqlx::Error>` - Created assignment or error
    pub async fn assign(pool: &PgPool, card_id: Uuid, label_id: Uuid) -> Result<Self, sqlx::Error> {
        let assignment = sqlx::query_as!(
            CardLabel,
            r#"
            INSERT INTO card_labels (card_id, label_id)
            VALUES ($1, $2)
            ON CONFLICT (card_id, label_id) DO NOTHING
            RETURNING card_id, label_id, created_at
            "#,
            card_id,
            label_id
        )
        .fetch_one(pool)
        .await?;

        Ok(assignment)
    }

    /// Unassign a label from a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    /// * `label_id` - Label UUID
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if unassigned, false if not found
    pub async fn unassign(
        pool: &PgPool,
        card_id: Uuid,
        label_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM card_labels
            WHERE card_id = $1 AND label_id = $2
            "#,
            card_id,
            label_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get all card-label assignments for a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `Result<Vec<CardLabel>, sqlx::Error>` - List of assignments
    pub async fn find_by_card_id(pool: &PgPool, card_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let assignments = sqlx::query_as!(
            CardLabel,
            r#"
            SELECT card_id, label_id, created_at
            FROM card_labels
            WHERE card_id = $1
            "#,
            card_id
        )
        .fetch_all(pool)
        .await?;

        Ok(assignments)
    }
}
