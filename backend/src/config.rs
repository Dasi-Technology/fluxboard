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
        }
    }
}
