use actix_web::{App, HttpServer, middleware, web};
use log::info;
use std::io;

mod config;
mod db;
mod error;
mod handlers;
mod models;
mod services;
mod utils;
mod websocket;

use config::Config;
use db::init_pool;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = Config::from_env();
    info!("Starting Fluxboard backend server...");
    info!(
        "Server will run on {}:{}",
        config.server_host, config.server_port
    );

    // Initialize database connection pool
    let pool = init_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    info!("Database connection pool established");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Share database pool across all handlers
            .app_data(web::Data::new(pool.clone()))
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // CORS middleware (will be configured properly later)
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // WebSocket endpoint placeholder
            .route("/ws", web::get().to(websocket_handler))
        // API routes will be added here
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .run()
    .await
}

/// Health check endpoint
async fn health_check() -> actix_web::Result<impl actix_web::Responder> {
    Ok(web::Json(serde_json::json!({
        "status": "ok",
        "message": "Fluxboard backend is running"
    })))
}

/// WebSocket handler placeholder
async fn websocket_handler(
    req: actix_web::HttpRequest,
    stream: web::Payload,
) -> actix_web::Result<impl actix_web::Responder> {
    // TODO: Implement WebSocket upgrade logic
    Ok(web::Json(serde_json::json!({
        "error": "WebSocket not yet implemented"
    })))
}
