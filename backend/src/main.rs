use actix_web::{App, HttpServer, middleware, web};
use log::info;
use std::io;
use std::sync::Arc;

mod config;
mod db;
mod error;
mod handlers;
mod models;
mod services;
mod sse;
mod utils;

use config::Config;
use db::init_pool;
use services::AiService;

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

    // Initialize SSE manager
    let sse_manager = Arc::new(sse::SseManager::new());
    info!("SSE manager initialized");

    // Initialize AI service if API key is configured
    let ai_service = config.gemini_api_key.clone().map(|key| {
        info!("AI service initialized with Gemini API");
        Arc::new(AiService::new(key))
    });

    // Start HTTP server
    let config_clone = config.clone();
    HttpServer::new(move || {
        // Configure CORS with localhost:3000 and optional additional origin
        let mut cors = actix_cors::Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::HeaderName::from_static("x-board-password"),
            ])
            .max_age(3600);

        // Add additional CORS origin if configured
        if let Some(ref origin) = config_clone.cors_origin {
            if !origin.is_empty() {
                info!("Adding additional CORS origin: {}", origin);
                cors = cors.allowed_origin(origin.as_str());
            }
        }

        let mut app = App::new()
            // Share database pool across all handlers
            .app_data(web::Data::new(pool.clone()))
            // Share SSE manager across all handlers
            .app_data(web::Data::new(sse_manager.clone()));

        // Add AI service if available
        if let Some(ref ai_svc) = ai_service {
            app = app.app_data(web::Data::new(ai_svc.clone()));
        }

        app
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // CORS middleware
            .wrap(cors)
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // Configure API routes (including SSE)
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
