use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{Card, Column, Label};

/// Board model representing a Kanban board
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Board {
    pub id: Uuid,
    pub share_token: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Board with all related data (columns, cards, labels)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardWithRelations {
    pub id: Uuid,
    pub share_token: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub columns: Vec<ColumnWithCards>,
}

/// Column with cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnWithCards {
    pub id: Uuid,
    pub board_id: Uuid,
    pub title: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cards: Vec<CardWithLabels>,
}

/// Card with labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardWithLabels {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub labels: Vec<Label>,
}

/// Input data for creating a new board
#[derive(Debug, Deserialize)]
pub struct CreateBoardInput {
    pub title: String,
    pub description: Option<String>,
}

/// Input data for updating a board
#[derive(Debug, Deserialize)]
pub struct UpdateBoardInput {
    pub title: Option<String>,
    pub description: Option<String>,
}

impl Board {
    /// Create a new board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `input` - Board creation data
    ///
    /// # Returns
    /// * `Result<Board, sqlx::Error>` - Created board or error
    pub async fn create(pool: &PgPool, input: CreateBoardInput) -> Result<Self, sqlx::Error> {
        let share_token = Self::generate_share_token();

        let board = sqlx::query_as!(
            Board,
            r#"
            INSERT INTO boards (share_token, title, description)
            VALUES ($1, $2, $3)
            RETURNING id, share_token, title, description, created_at, updated_at
            "#,
            share_token,
            input.title,
            input.description
        )
        .fetch_one(pool)
        .await?;

        Ok(board)
    }

    /// Find a board by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Board UUID
    ///
    /// # Returns
    /// * `Result<Option<Board>, sqlx::Error>` - Found board or None
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let board = sqlx::query_as!(
            Board,
            r#"
            SELECT id, share_token, title, description, created_at, updated_at
            FROM boards
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(board)
    }

    /// Find a board by share token
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `share_token` - Unique share token
    ///
    /// # Returns
    /// * `Result<Option<Board>, sqlx::Error>` - Found board or None
    pub async fn find_by_share_token(
        pool: &PgPool,
        share_token: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let board = sqlx::query_as!(
            Board,
            r#"
            SELECT id, share_token, title, description, created_at, updated_at
            FROM boards
            WHERE share_token = $1
            "#,
            share_token
        )
        .fetch_optional(pool)
        .await?;

        Ok(board)
    }

    /// Find a board by share token with all relations
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `share_token` - Unique share token
    ///
    /// # Returns
    /// * `Result<Option<BoardWithRelations>, sqlx::Error>` - Found board with relations or None
    pub async fn find_by_share_token_with_relations(
        pool: &PgPool,
        share_token: &str,
    ) -> Result<Option<BoardWithRelations>, sqlx::Error> {
        // First get the board
        let board = Self::find_by_share_token(pool, share_token).await?;

        let board = match board {
            Some(b) => b,
            None => return Ok(None),
        };

        // Get all columns for this board
        let columns = Column::find_by_board_id(pool, board.id).await?;

        // Build columns with cards
        let mut columns_with_cards = Vec::new();
        for column in columns {
            // Get all cards for this column
            let cards = Card::find_by_column_id(pool, column.id).await?;

            // Build cards with labels
            let mut cards_with_labels = Vec::new();
            for card in cards {
                // Get all labels for this card
                let labels = Label::find_by_card_id(pool, card.id).await?;

                cards_with_labels.push(CardWithLabels {
                    id: card.id,
                    column_id: card.column_id,
                    title: card.title,
                    description: card.description,
                    position: card.position,
                    created_at: card.created_at,
                    updated_at: card.updated_at,
                    labels,
                });
            }

            columns_with_cards.push(ColumnWithCards {
                id: column.id,
                board_id: column.board_id,
                title: column.title,
                position: column.position,
                created_at: column.created_at,
                updated_at: column.updated_at,
                cards: cards_with_labels,
            });
        }

        Ok(Some(BoardWithRelations {
            id: board.id,
            share_token: board.share_token,
            title: board.title,
            description: board.description,
            created_at: board.created_at,
            updated_at: board.updated_at,
            columns: columns_with_cards,
        }))
    }

    /// List all boards
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Result<Vec<Board>, sqlx::Error>` - List of all boards
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let boards = sqlx::query_as!(
            Board,
            r#"
            SELECT id, share_token, title, description, created_at, updated_at
            FROM boards
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

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
    /// * `Result<Option<Board>, sqlx::Error>` - Updated board or None if not found
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateBoardInput,
    ) -> Result<Option<Self>, sqlx::Error> {
        let board = sqlx::query_as!(
            Board,
            r#"
            UPDATE boards
            SET 
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, share_token, title, description, created_at, updated_at
            "#,
            id,
            input.title,
            input.description
        )
        .fetch_optional(pool)
        .await?;

        Ok(board)
    }

    /// Delete a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Board UUID
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if deleted, false if not found
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM boards
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Generate a unique share token
    ///
    /// # Returns
    /// * `String` - Random alphanumeric share token
    fn generate_share_token() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const TOKEN_LEN: usize = 12;

        let mut rng = rand::thread_rng();
        (0..TOKEN_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}
