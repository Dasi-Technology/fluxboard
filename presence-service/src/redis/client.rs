//! Redis client implementation with connection pooling and health checks.

use redis::aio::ConnectionManager;
use redis::Client;
use thiserror::Error;
use tracing::{error, info};

/// Errors that can occur during Redis operations
#[derive(Debug, Error)]
pub enum RedisError {
    #[error("Redis connection error: {0}")]
    ConnectionError(#[from] redis::RedisError),

    #[error("Redis URL parse error: {0}")]
    UrlParseError(String),

    #[error("Redis health check failed: {0}")]
    HealthCheckFailed(String),
}

/// Redis client with connection pooling
#[derive(Clone)]
pub struct RedisClient {
    client: Client,
    connection_manager: ConnectionManager,
}

impl RedisClient {
    /// Create a new Redis client with connection pooling
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Returns
    ///
    /// A `Result` containing the `RedisClient` or a `RedisError` if connection fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use presence_service::redis::client::RedisClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let redis_url = "redis://localhost:6379";
    /// let client = RedisClient::new(redis_url).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        info!("Connecting to Redis at {}", redis_url);

        // Create Redis client
        let client =
            Client::open(redis_url).map_err(|e| RedisError::UrlParseError(e.to_string()))?;

        // Create connection manager for connection pooling
        let connection_manager = ConnectionManager::new(client.clone())
            .await
            .map_err(RedisError::ConnectionError)?;

        info!("Successfully connected to Redis");

        let redis_client = Self {
            client,
            connection_manager,
        };

        // Perform initial health check
        redis_client.health_check().await?;

        Ok(redis_client)
    }

    /// Get a connection from the pool
    ///
    /// # Returns
    ///
    /// A `Result` containing a `ConnectionManager` or a `RedisError`
    pub async fn get_connection(&self) -> Result<ConnectionManager, RedisError> {
        Ok(self.connection_manager.clone())
    }

    /// Perform a health check to verify Redis connectivity
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `RedisError` if the health check fails
    pub async fn health_check(&self) -> Result<(), RedisError> {
        use redis::AsyncCommands;

        let mut conn = self.get_connection().await?;

        // Try to ping Redis
        let pong: String = conn
            .ping()
            .await
            .map_err(|e| RedisError::HealthCheckFailed(e.to_string()))?;

        if pong != "PONG" {
            return Err(RedisError::HealthCheckFailed(format!(
                "Expected PONG, got {}",
                pong
            )));
        }

        info!("Redis health check passed");
        Ok(())
    }

    /// Get the underlying Redis client
    ///
    /// This is useful for operations that need direct access to the client
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_redis_connection() {
        let redis_url = "redis://localhost:6379";
        let client = RedisClient::new(redis_url).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_health_check() {
        let redis_url = "redis://localhost:6379";
        let client = RedisClient::new(redis_url).await.unwrap();
        let result = client.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let redis_url = "invalid://url";
        let client = RedisClient::new(redis_url).await;
        assert!(client.is_err());
    }
}
