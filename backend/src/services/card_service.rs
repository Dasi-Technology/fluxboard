use crate::error::{AppError, AppResult};
use crate::models::{Card, CreateCardInput, UpdateCardInput};
use sqlx::PgPool;
use uuid::Uuid;

/// Service for card-related business logic
pub struct CardService;

impl CardService {
    /// Create a new card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `column_id` - Column UUID
    /// * `title` - Card title
    /// * `description` - Optional card description
    /// * `position` - Card position
    ///
    /// # Returns
    /// * `AppResult<Card>` - Created card or error
    pub async fn create_card(
        pool: &PgPool,
        column_id: Uuid,
        title: String,
        description: Option<String>,
        position: i32,
    ) -> AppResult<Card> {
        // Validate input
        if title.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Card title cannot be empty".to_string(),
            ));
        }

        if title.len() > 255 {
            return Err(AppError::BadRequest(
                "Card title cannot exceed 255 characters".to_string(),
            ));
        }

        if position < 0 {
            return Err(AppError::BadRequest(
                "Card position cannot be negative".to_string(),
            ));
        }

        let input = CreateCardInput {
            column_id,
            title,
            description,
            position,
        };

        let card = Card::create(pool, input).await?;
        Ok(card)
    }

    /// Get card by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Card UUID
    ///
    /// # Returns
    /// * `AppResult<Card>` - Found card or error
    pub async fn get_card_by_id(pool: &PgPool, id: Uuid) -> AppResult<Card> {
        Card::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Card with ID {} not found", id)))
    }

    /// Get all cards for a column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `column_id` - Column UUID
    ///
    /// # Returns
    /// * `AppResult<Vec<Card>>` - List of cards ordered by position
    pub async fn get_cards_by_column_id(pool: &PgPool, column_id: Uuid) -> AppResult<Vec<Card>> {
        let cards = Card::find_by_column_id(pool, column_id).await?;
        Ok(cards)
    }

    /// Get all cards for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `AppResult<Vec<Card>>` - List of all cards in the board
    pub async fn get_cards_by_board_id(pool: &PgPool, board_id: Uuid) -> AppResult<Vec<Card>> {
        let cards = Card::find_by_board_id(pool, board_id).await?;
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
    /// * `AppResult<Card>` - Updated card or error
    pub async fn update_card(pool: &PgPool, id: Uuid, input: UpdateCardInput) -> AppResult<Card> {
        // Validate title if provided
        if let Some(ref title) = input.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Card title cannot be empty".to_string(),
                ));
            }
            if title.len() > 255 {
                return Err(AppError::BadRequest(
                    "Card title cannot exceed 255 characters".to_string(),
                ));
            }
        }

        // Validate position if provided
        if let Some(position) = input.position {
            if position < 0 {
                return Err(AppError::BadRequest(
                    "Card position cannot be negative".to_string(),
                ));
            }
        }

        Card::update(pool, id, input)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Card with ID {} not found", id)))
    }

    /// Delete a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Card UUID
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn delete_card(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let deleted = Card::delete(pool, id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Card with ID {} not found", id)))
        }
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
    /// * `AppResult<Card>` - Updated card or error
    pub async fn move_card(
        pool: &PgPool,
        id: Uuid,
        new_column_id: Uuid,
        new_position: i32,
    ) -> AppResult<Card> {
        // Validate position
        if new_position < 0 {
            return Err(AppError::BadRequest(
                "Card position cannot be negative".to_string(),
            ));
        }

        Card::move_to_column(pool, id, new_column_id, new_position)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Card with ID {} not found", id)))
    }

    /// Reorder cards within a column
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `column_id` - Column UUID
    /// * `card_positions` - Vec of (card_id, new_position) tuples
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn reorder_cards(
        pool: &PgPool,
        column_id: Uuid,
        card_positions: Vec<(Uuid, i32)>,
    ) -> AppResult<()> {
        // Validate positions
        for (_, position) in &card_positions {
            if *position < 0 {
                return Err(AppError::BadRequest(
                    "Card position cannot be negative".to_string(),
                ));
            }
        }

        Card::reorder(pool, column_id, card_positions).await?;
        Ok(())
    }
}
