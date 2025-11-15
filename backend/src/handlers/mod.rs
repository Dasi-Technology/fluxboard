//! HTTP handlers module
//!
//! This module contains all HTTP request handlers for the REST API.
//! Handlers are organized by resource type.

pub mod board_handlers;
pub mod card_handlers;
pub mod column_handlers;
pub mod label_handlers;

use actix_web::web;

/// Configure all API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Board routes
            .route("/boards", web::post().to(board_handlers::create_board))
            .route("/boards", web::get().to(board_handlers::list_boards))
            .route("/boards/{id}", web::get().to(board_handlers::get_board))
            .route("/boards/{id}", web::put().to(board_handlers::update_board))
            .route(
                "/boards/{id}",
                web::delete().to(board_handlers::delete_board),
            )
            .route(
                "/boards/share/{token}",
                web::get().to(board_handlers::get_board_by_share_token),
            )
            .route(
                "/boards/share/{token}",
                web::put().to(board_handlers::update_board_by_share_token),
            )
            // Column routes
            .route(
                "/boards/{board_id}/columns",
                web::post().to(column_handlers::create_column),
            )
            .route(
                "/boards/{board_id}/columns/reorder",
                web::patch().to(column_handlers::reorder_columns),
            )
            .route(
                "/columns/{id}",
                web::put().to(column_handlers::update_column),
            )
            .route(
                "/columns/{id}",
                web::delete().to(column_handlers::delete_column),
            )
            // Card routes
            .route(
                "/columns/{column_id}/cards",
                web::post().to(card_handlers::create_card),
            )
            .route(
                "/columns/{column_id}/cards/reorder",
                web::patch().to(card_handlers::reorder_cards),
            )
            .route("/cards/{id}", web::get().to(card_handlers::get_card))
            .route("/cards/{id}", web::put().to(card_handlers::update_card))
            .route("/cards/{id}", web::delete().to(card_handlers::delete_card))
            .route(
                "/cards/{id}/move",
                web::patch().to(card_handlers::move_card),
            )
            // Label routes
            .route(
                "/cards/{card_id}/labels",
                web::post().to(label_handlers::create_label),
            )
            .route("/labels/{id}", web::put().to(label_handlers::update_label))
            .route(
                "/labels/{id}",
                web::delete().to(label_handlers::delete_label),
            ),
    );
}
