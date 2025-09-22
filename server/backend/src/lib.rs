

pub mod api;
pub mod blockchain;
pub mod game;

use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

use crate::api::handlers;
use crate::game::GameState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize shared game state
    let game_state = Arc::new(RwLock::new(GameState::new()));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        // Core game endpoints
        .route("/shuffle-and-deal", post(handlers::shuffle_and_deal))
        .route("/player/:player_id/hand", get(handlers::get_initial_hands))
        .route(
            "/player/:player_id/hand-js",
            get(handlers::get_initial_hands_js),
        )
        .route("/draw-card", post(handlers::draw_card))
        .route("/draw-multiple", post(handlers::draw_multiple_cards))
        .route("/play-card", post(handlers::play_card))
        // Game state endpoints
        .route("/game-state", get(handlers::get_game_state))
        .route("/game-status", get(handlers::get_game_status))
        .route("/top-discard", get(handlers::get_top_discard_card))
        // Utility endpoints
        .route("/reset", post(handlers::reset_game))
        .route(
            "/player/:player_id/hand-count",
            get(handlers::get_player_hand_count),
        )
        .route("/health", get(handlers::health_check))
        .with_state(game_state)
        .layer(cors);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("UNO backend server listening on 0.0.0.0:3001");

    axum::serve(listener, app).await?;
    Ok(())
}
