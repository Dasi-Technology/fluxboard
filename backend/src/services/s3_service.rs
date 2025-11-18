use crate::config::Config;
use crate::error::{AppError, AppResult};
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Service for S3-related operations
#[derive(Clone)]
pub struct S3Service {
    client: Arc<S3Client>,
    bucket: String,
    upload_url_expiry_minutes: i64,
    download_url_expiry_days: i64,
}

impl S3Service {
    /// Create a new S3 service instance
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// * `AppResult<S3Service>` - New service instance or error
    pub async fn new(config: &Config) -> AppResult<Self> {
        // Configure AWS region
        let region_provider =
            RegionProviderChain::first_try(aws_config::Region::new(config.aws_region.clone()));

        // Build AWS config
        let mut aws_config_builder =
            aws_config::defaults(BehaviorVersion::latest()).region(region_provider);

        // Use explicit credentials if provided, otherwise use default credentials chain
        if let (Some(access_key_id), Some(secret_access_key)) =
            (&config.aws_access_key_id, &config.aws_secret_access_key)
        {
            aws_config_builder =
                aws_config_builder.credentials_provider(aws_sdk_s3::config::Credentials::new(
                    access_key_id,
                    secret_access_key,
                    None, // session token
                    None, // expiry
                    "env",
                ));
        }

        let aws_config = aws_config_builder.load().await;
        let client = S3Client::new(&aws_config);

        Ok(Self {
            client: Arc::new(client),
            bucket: config.aws_s3_bucket.clone(),
            upload_url_expiry_minutes: config.s3_upload_url_expiry_minutes,
            download_url_expiry_days: config.s3_download_url_expiry_days,
        })
    }

    /// Generate a pre-signed PUT URL for uploading a file
    ///
    /// # Arguments
    /// * `s3_key` - S3 object key
    /// * `content_type` - MIME type
    ///
    /// # Returns
    /// * `AppResult<String>` - Pre-signed URL or error
    pub async fn generate_upload_url(&self, s3_key: &str, content_type: &str) -> AppResult<String> {
        log::info!(
            "Generating pre-signed upload URL for s3_key: {}, content_type: {}",
            s3_key,
            content_type
        );
        log::info!("S3 bucket: {}, region configured", self.bucket);

        let expiry_duration = Duration::from_secs((self.upload_url_expiry_minutes * 60) as u64);

        let presigning_config = PresigningConfig::builder()
            .expires_in(expiry_duration)
            .build()
            .map_err(|e| {
                log::error!("Failed to build presigning config: {}", e);
                AppError::InternalError(format!("Failed to build presigning config: {}", e))
            })?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                log::error!("Failed to generate upload URL: {}", e);
                AppError::InternalError(format!("Failed to generate upload URL: {}", e))
            })?;

        let url = presigned_request.uri().to_string();
        log::info!(
            "Generated pre-signed URL (first 100 chars): {}",
            &url[..url.len().min(100)]
        );

        Ok(url)
    }

    /// Generate a pre-signed GET URL for downloading a file
    ///
    /// # Arguments
    /// * `s3_key` - S3 object key
    ///
    /// # Returns
    /// * `AppResult<String>` - Pre-signed URL or error
    pub async fn generate_download_url(&self, s3_key: &str) -> AppResult<String> {
        let expiry_duration = Duration::from_secs((self.download_url_expiry_days * 86400) as u64);

        let presigning_config = PresigningConfig::builder()
            .expires_in(expiry_duration)
            .build()
            .map_err(|e| {
                AppError::InternalError(format!("Failed to build presigning config: {}", e))
            })?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to generate download URL: {}", e))
            })?;

        Ok(presigned_request.uri().to_string())
    }

    /// Verify that an S3 object exists
    ///
    /// # Arguments
    /// * `s3_key` - S3 object key
    ///
    /// # Returns
    /// * `AppResult<bool>` - True if object exists, false otherwise
    pub async fn verify_object_exists(&self, s3_key: &str) -> AppResult<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                // Check if it's a "not found" error
                if e.to_string().contains("NotFound") || e.to_string().contains("404") {
                    Ok(false)
                } else {
                    Err(AppError::InternalError(format!(
                        "Failed to verify object existence: {}",
                        e
                    )))
                }
            }
        }
    }

    /// Delete an S3 object
    ///
    /// # Arguments
    /// * `s3_key` - S3 object key
    ///
    /// # Returns
    /// * `AppResult<()>` - Success or error
    pub async fn delete_object(&self, s3_key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to delete S3 object: {}", e)))?;

        Ok(())
    }

    /// Build an S3 key for an attachment
    ///
    /// # Arguments
    /// * `card_id` - Card UUID
    /// * `attachment_id` - Attachment UUID
    /// * `extension` - File extension
    ///
    /// # Returns
    /// * `String` - S3 object key in format: attachments/{card_id}/{attachment_id}.{ext}
    pub fn build_s3_key(card_id: Uuid, attachment_id: Uuid, extension: &str) -> String {
        format!("attachments/{}/{}.{}", card_id, attachment_id, extension)
    }

    /// Extract file extension from filename
    ///
    /// # Arguments
    /// * `filename` - Original filename
    ///
    /// # Returns
    /// * `String` - File extension or "bin" if none found
    pub fn extract_extension(filename: &str) -> String {
        filename
            .rsplit('.')
            .next()
            .filter(|ext| !ext.is_empty() && ext.len() <= 10)
            .unwrap_or("bin")
            .to_lowercase()
    }
}
