use crate::error::{AppError, AppResult};
use crate::models::{BoardLabel, CardLabel, CreateBoardLabelInput, UpdateBoardLabelInput};
use sqlx::PgPool;
use uuid::Uuid;

/// Service for board label-related business logic
pub struct BoardLabelService;

impl BoardLabelService {
    /// Create a new board label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    /// * `name` - Label name
    /// * `color` - Label color (hex)
    ///
    /// # Returns
    /// * `AppResult<BoardLabel>` - Created label or error
    pub async fn create_label(
        pool: &PgPool,
        board_id: Uuid,
        name: String,
        color: String,
    ) -> AppResult<BoardLabel> {
        // Validate input
        if name.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Label name cannot be empty".to_string(),
            ));
        }

        if name.len() > 50 {
            return Err(AppError::BadRequest(
                "Label name cannot exceed 50 characters".to_string(),
            ));
        }

        if color.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Label color cannot be empty".to_string(),
            ));
        }

        if color.len() > 20 {
            return Err(AppError::BadRequest(
                "Label color cannot exceed 20 characters".to_string(),
            ));
        }

        let input = CreateBoardLabelInput {
            board_id,
            name,
            color,
        };

        let label = BoardLabel::create(pool, input).await?;
        Ok(label)
    }

    /// Get label by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    ///
    /// # Returns
    /// * `AppResult<BoardLabel>` - Found label or error
    pub async fn get_label_by_id(pool: &PgPool, id: Uuid) -> AppResult<BoardLabel> {
        BoardLabel::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Label with ID {} not found", id)))
    }

    /// Get all labels for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `AppResult<Vec<BoardLabel>>` - List of labels for the board
    pub async fn get_labels_by_board_id(
        pool: &PgPool,
        board_id: Uuid,
    ) -> AppResult<Vec<BoardLabel>> {
        let labels = BoardLabel::find_by_board_id(pool, board_id).await?;
        Ok(labels)
    }

    /// Get all labels assigned to a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `AppResult<Vec<BoardLabel>>` - List of labels assigned to the card
    pub async fn get_labels_by_card_id(pool: &PgPool, card_id: Uuid) -> AppResult<Vec<BoardLabel>> {
        let labels = BoardLabel::find_by_card_id(pool, card_id).await?;
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
    /// * `AppResult<BoardLabel>` - Updated label or error
    pub async fn update_label(
        pool: &PgPool,
        id: Uuid,
        input: UpdateBoardLabelInput,
    ) -> AppResult<BoardLabel> {
        // Validate name if provided
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Label name cannot be empty".to_string(),
                ));
            }
            if name.len() > 50 {
                return Err(AppError::BadRequest(
                    "Label name cannot exceed 50 characters".to_string(),
                ));
            }
        }

        // Validate color if provided
        if let Some(ref color) = input.color {
            if color.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Label color cannot be empty".to_string(),
                ));
            }
            if color.len() > 20 {
                return Err(AppError::BadRequest(
                    "Label color cannot exceed 20 characters".to_string(),
                ));
            }
        }

        BoardLabel::update(pool, id, input)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Label with ID {} not found", id)))
    }

    /// Delete a board label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn delete_label(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let deleted = BoardLabel::delete(pool, id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "Label with ID {} not found",
                id
            )))
        }
    }

    /// Assign a label to a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    /// * `label_id` - Label UUID
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn assign_label_to_card(
        pool: &PgPool,
        card_id: Uuid,
        label_id: Uuid,
    ) -> AppResult<()> {
        CardLabel::assign(pool, card_id, label_id).await?;
        Ok(())
    }

    /// Unassign a label from a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    /// * `label_id` - Label UUID
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn unassign_label_from_card(
        pool: &PgPool,
        card_id: Uuid,
        label_id: Uuid,
    ) -> AppResult<()> {
        let unassigned = CardLabel::unassign(pool, card_id, label_id).await?;
        if unassigned {
            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "Label assignment not found for card {} and label {}",
                card_id, label_id
            )))
        }
    }
}
