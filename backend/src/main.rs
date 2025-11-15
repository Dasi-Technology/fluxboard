use actix::Actor;
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
use websocket::WsServer;

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

    // Run database migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    info!("Database migrations completed successfully");

    // Start WebSocket server actor
    let ws_server = WsServer::new().start();
    info!("WebSocket server started");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Share database pool across all handlers
            .app_data(web::Data::new(pool.clone()))
            // Share WebSocket server across all handlers
            .app_data(web::Data::new(ws_server.clone()))
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // CORS middleware for development
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // WebSocket endpoint
            .route("/ws/{share_token}", web::get().to(websocket::ws_handler))
            // Configure API routes
            .configure(handlers::configure_routes)
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
