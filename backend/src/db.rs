use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Initialize PostgreSQL connection pool
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
///
/// # Returns
/// * `Result<PgPool, sqlx::Error>` - Connection pool or error
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5) // Maximum number of connections in the pool
        .acquire_timeout(Duration::from_secs(3)) // Timeout for acquiring a connection
        .connect(database_url)
        .await
}

/// Test database connection
///
/// # Arguments
/// * `pool` - Reference to the database connection pool
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Ok if connection is successful
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    Ok(())
}
