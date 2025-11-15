use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

mod config;
mod connection;
mod handlers;
mod presence;
mod protocol;
mod redis;
mod utils;

use connection::manager::ConnectionManager;
use handlers::websocket::handle_connection;
use redis::client::RedisClient;
use redis::pubsub::RedisPubSub;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting presence-service...");

    // Load config from .env file
    dotenvy::dotenv().ok();

    // Get WebSocket port from environment or use default
    let port = std::env::var("WS_PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Initialize Redis
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    info!("Connecting to Redis at {}", redis_url);
    let redis_client = RedisClient::new(&redis_url).await?;
    let redis_pubsub = Arc::new(RedisPubSub::new(redis_client).await?);
    info!("Redis connection established");

    // Create connection manager with Redis support
    let manager = Arc::new(ConnectionManager::new(Arc::clone(&redis_pubsub)));

    // Start Redis listener for cross-instance coordination
    let manager_clone = Arc::clone(&manager);
    tokio::spawn(async move {
        manager_clone.start_redis_listener().await;
    });

    // Bind TCP listener
    let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on {}", addr);

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let manager = Arc::clone(&manager);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, peer_addr, manager).await {
                        tracing::error!("Connection error for {}: {}", peer_addr, e);
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    }
}
