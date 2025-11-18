use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

/// Card attachment model representing an image attachment stored in S3
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CardAttachment {
    pub id: Uuid,
    pub card_id: Uuid,
    pub uploaded_by: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub content_type: String,
    pub file_size: i32,
    pub s3_key: String,
    pub s3_bucket: String,
    pub is_confirmed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to generate a pre-signed upload URL
#[derive(Debug, Deserialize, Validate)]
pub struct UploadUrlRequest {
    #[validate(length(min = 1, max = 255))]
    pub filename: String,
    #[validate(length(min = 1, max = 100))]
    pub content_type: String,
    #[validate(range(min = 1))]
    pub file_size: i32,
}

/// Response containing pre-signed upload URL
#[derive(Debug, Serialize)]
pub struct UploadUrlResponse {
    pub upload_url: String,
    pub attachment_id: Uuid,
    pub s3_key: String,
}

impl CardAttachment {
    /// Create a new attachment record (unconfirmed) with a pre-generated ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Pre-generated attachment UUID
    /// * `card_id` - Card UUID
    /// * `uploaded_by` - User UUID who is uploading
    /// * `filename` - Generated filename
    /// * `original_filename` - Original filename from user
    /// * `content_type` - MIME type
    /// * `file_size` - File size in bytes
    /// * `s3_key` - S3 object key
    /// * `s3_bucket` - S3 bucket name
    ///
    /// # Returns
    /// * `Result<CardAttachment, sqlx::Error>` - Created attachment record
    pub async fn create_with_id(
        pool: &PgPool,
        id: Uuid,
        card_id: Uuid,
        uploaded_by: Uuid,
        filename: String,
        original_filename: String,
        content_type: String,
        file_size: i32,
        s3_key: String,
        s3_bucket: String,
    ) -> Result<Self, sqlx::Error> {
        let attachment = sqlx::query_as!(
            CardAttachment,
            r#"
            INSERT INTO card_attachments
                (id, card_id, uploaded_by, filename, original_filename, content_type,
                 file_size, s3_key, s3_bucket, is_confirmed)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false)
            RETURNING id, card_id, uploaded_by, filename, original_filename,
                      content_type, file_size, s3_key, s3_bucket,
                      is_confirmed as "is_confirmed!",
                      created_at as "created_at!",
                      updated_at as "updated_at!"
            "#,
            id,
            card_id,
            uploaded_by,
            filename,
            original_filename,
            content_type,
            file_size,
            s3_key,
            s3_bucket
        )
        .fetch_one(pool)
        .await?;

        Ok(attachment)
    }

    /// Create a new attachment record (unconfirmed)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    /// * `uploaded_by` - User UUID who is uploading
    /// * `filename` - Generated filename
    /// * `original_filename` - Original filename from user
    /// * `content_type` - MIME type
    /// * `file_size` - File size in bytes
    /// * `s3_key` - S3 object key
    /// * `s3_bucket` - S3 bucket name
    ///
    /// # Returns
    /// * `Result<CardAttachment, sqlx::Error>` - Created attachment record
    #[allow(dead_code)]
    pub async fn create(
        pool: &PgPool,
        card_id: Uuid,
        uploaded_by: Uuid,
        filename: String,
        original_filename: String,
        content_type: String,
        file_size: i32,
        s3_key: String,
        s3_bucket: String,
    ) -> Result<Self, sqlx::Error> {
        let attachment = sqlx::query_as!(
            CardAttachment,
            r#"
            INSERT INTO card_attachments
                (card_id, uploaded_by, filename, original_filename, content_type,
                 file_size, s3_key, s3_bucket, is_confirmed)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)
            RETURNING id, card_id, uploaded_by, filename, original_filename,
                      content_type, file_size, s3_key, s3_bucket,
                      is_confirmed as "is_confirmed!",
                      created_at as "created_at!",
                      updated_at as "updated_at!"
            "#,
            card_id,
            uploaded_by,
            filename,
            original_filename,
            content_type,
            file_size,
            s3_key,
            s3_bucket
        )
        .fetch_one(pool)
        .await?;

        Ok(attachment)
    }

    /// Find an attachment by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Attachment UUID
    ///
    /// # Returns
    /// * `Result<Option<CardAttachment>, sqlx::Error>` - Found attachment or None
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let attachment = sqlx::query_as!(
            CardAttachment,
            r#"
            SELECT id, card_id, uploaded_by, filename, original_filename,
                   content_type, file_size, s3_key, s3_bucket,
                   is_confirmed as "is_confirmed!",
                   created_at as "created_at!",
                   updated_at as "updated_at!"
            FROM card_attachments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(attachment)
    }

    /// Find all attachments for a card
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `card_id` - Card UUID
    ///
    /// # Returns
    /// * `Result<Vec<CardAttachment>, sqlx::Error>` - List of attachments
    pub async fn find_by_card_id(pool: &PgPool, card_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let attachments = sqlx::query_as!(
            CardAttachment,
            r#"
            SELECT id, card_id, uploaded_by, filename, original_filename,
                   content_type, file_size, s3_key, s3_bucket,
                   is_confirmed as "is_confirmed!",
                   created_at as "created_at!",
                   updated_at as "updated_at!"
            FROM card_attachments
            WHERE card_id = $1 AND is_confirmed = true
            ORDER BY created_at ASC
            "#,
            card_id
        )
        .fetch_all(pool)
        .await?;

        Ok(attachments)
    }

    /// Confirm an attachment (mark as confirmed after successful S3 upload)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Attachment UUID
    ///
    /// # Returns
    /// * `Result<Option<CardAttachment>, sqlx::Error>` - Updated attachment or None
    pub async fn confirm(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let attachment = sqlx::query_as!(
            CardAttachment,
            r#"
            UPDATE card_attachments
            SET is_confirmed = true, updated_at = NOW()
            WHERE id = $1
            RETURNING id, card_id, uploaded_by, filename, original_filename,
                      content_type, file_size, s3_key, s3_bucket,
                      is_confirmed as "is_confirmed!",
                      created_at as "created_at!",
                      updated_at as "updated_at!"
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(attachment)
    }

    /// Delete an attachment
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Attachment UUID
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if deleted, false if not found
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM card_attachments
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
