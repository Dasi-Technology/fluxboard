use std::env;

/// Application configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct Config {
    /// PostgreSQL database connection URL
    pub database_url: String,
    /// Server host address
    pub server_host: String,
    /// Server port number
    pub server_port: u16,
    /// Logging level configuration
    pub rust_log: String,
    /// Additional CORS allowed origin (optional, for production)
    pub cors_origin: Option<String>,
    /// Gemini API key for AI features
    pub gemini_api_key: Option<String>,
    /// JWT secret key for token signing
    pub jwt_secret: String,
    /// Access token expiry in seconds (default: 900 = 15 minutes)
    pub jwt_access_token_expiry: i64,
    /// Refresh token expiry in seconds (default: 2592000 = 30 days)
    pub jwt_refresh_token_expiry: i64,
    /// AWS region for S3
    pub aws_region: String,
    /// AWS S3 bucket name
    pub aws_s3_bucket: String,
    /// AWS access key ID (optional, uses default credentials if not set)
    pub aws_access_key_id: Option<String>,
    /// AWS secret access key (optional, uses default credentials if not set)
    pub aws_secret_access_key: Option<String>,
    /// Maximum upload file size in bytes (default: 5242880 = 5MB)
    pub s3_upload_max_size: i64,
    /// Allowed MIME types for uploads (comma-separated)
    pub s3_allowed_types: String,
    /// Pre-signed upload URL expiry in minutes (default: 15)
    pub s3_upload_url_expiry_minutes: i64,
    /// Pre-signed download URL expiry in days (default: 7)
    pub s3_download_url_expiry_days: i64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            cors_origin: env::var("CORS_ORIGIN").ok(),
            gemini_api_key: env::var("GEMINI_API_KEY").ok(),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_access_token_expiry: env::var("JWT_ACCESS_TOKEN_EXPIRY")
                .unwrap_or_else(|_| "900".to_string())
                .parse()
                .expect("JWT_ACCESS_TOKEN_EXPIRY must be a valid i64"),
            jwt_refresh_token_expiry: env::var("JWT_REFRESH_TOKEN_EXPIRY")
                .unwrap_or_else(|_| "2592000".to_string())
                .parse()
                .expect("JWT_REFRESH_TOKEN_EXPIRY must be a valid i64"),
            aws_region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            aws_s3_bucket: env::var("AWS_S3_BUCKET").expect("AWS_S3_BUCKET must be set"),
            aws_access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
            aws_secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            s3_upload_max_size: env::var("S3_UPLOAD_MAX_SIZE")
                .unwrap_or_else(|_| "5242880".to_string())
                .parse()
                .expect("S3_UPLOAD_MAX_SIZE must be a valid i64"),
            s3_allowed_types: env::var("S3_ALLOWED_TYPES")
                .unwrap_or_else(|_| "image/jpeg,image/png,image/gif,image/webp".to_string()),
            s3_upload_url_expiry_minutes: env::var("S3_UPLOAD_URL_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .expect("S3_UPLOAD_URL_EXPIRY_MINUTES must be a valid i64"),
            s3_download_url_expiry_days: env::var("S3_DOWNLOAD_URL_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .expect("S3_DOWNLOAD_URL_EXPIRY_DAYS must be a valid i64"),
        }
    }
}
