use crate::error::{AppError, AppResult};
use crate::models::{CreateLabelInput, Label, UpdateLabelInput};
use sqlx::PgPool;
use uuid::Uuid;

/// Service for label-related business logic
pub struct LabelService;

impl LabelService {
    /// Create a new label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    /// * `name` - Label name
    /// * `color` - Label color (hex or name)
    ///
    /// # Returns
    /// * `AppResult<Label>` - Created label or error
    pub async fn create_label(
        pool: &PgPool,
        card_id: Uuid,
        name: String,
        color: String,
    ) -> AppResult<Label> {
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

        let input = CreateLabelInput {
            card_id,
            name,
            color,
        };

        let label = Label::create(pool, input).await?;
        Ok(label)
    }

    /// Get label by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    ///
    /// # Returns
    /// * `AppResult<Label>` - Found label or error
    pub async fn get_label_by_id(pool: &PgPool, id: Uuid) -> AppResult<Label> {
        Label::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Label with ID {} not found", id)))
    }

    /// Get all labels for a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `AppResult<Vec<Label>>` - List of labels for the card
    pub async fn get_labels_by_card_id(pool: &PgPool, card_id: Uuid) -> AppResult<Vec<Label>> {
        let labels = Label::find_by_card_id(pool, card_id).await?;
        Ok(labels)
    }

    /// Get all labels for a board
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `board_id` - Board UUID
    ///
    /// # Returns
    /// * `AppResult<Vec<Label>>` - List of all labels in the board
    pub async fn get_labels_by_board_id(pool: &PgPool, board_id: Uuid) -> AppResult<Vec<Label>> {
        let labels = Label::find_by_board_id(pool, board_id).await?;
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
    /// * `AppResult<Label>` - Updated label or error
    pub async fn update_label(
        pool: &PgPool,
        id: Uuid,
        input: UpdateLabelInput,
    ) -> AppResult<Label> {
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

        Label::update(pool, id, input)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Label with ID {} not found", id)))
    }

    /// Delete a label
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Label UUID
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn delete_label(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let deleted = Label::delete(pool, id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "Label with ID {} not found",
                id
            )))
        }
    }

    /// Delete all labels for a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `AppResult<u64>` - Number of labels deleted
    pub async fn delete_labels_by_card_id(pool: &PgPool, card_id: Uuid) -> AppResult<u64> {
        let count = Label::delete_by_card_id(pool, card_id).await?;
        Ok(count)
    }
}
