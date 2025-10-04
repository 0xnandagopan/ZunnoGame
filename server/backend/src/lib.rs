pub mod api;
pub mod blockchain;
pub mod game;

use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

use crate::api::handlers;
use crate::game::GameState;

pub type SharedGames = Arc<RwLock<HashMap<String, GameState>>>;

pub async fn run() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize container for multiple concurrent games
    let games: SharedGames = Arc::new(RwLock::new(HashMap::new()));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Build the router. All game endpoints are namespaced by :game_id so that
    // multiple rooms can be hosted on the same server concurrently. Clients
    // can start a fresh game by calling the shuffle-and-deal endpoint with a
    // new game_id and then continue to interact with that id for subsequent
    // moves.
    let app = Router::new()
        // Core game endpoints
        .route(
            "/game/:game_id/shuffle-and-deal",
            post(handlers::shuffle_and_deal),
        )
        .route(
            "/game/:game_id/player/:player_id/hand",
            get(handlers::get_initial_hands),
        )
        .route(
            "/game/:game_id/player/:player_id/hand-js",
            get(handlers::get_initial_hands_js),
        )
        .route("/game/:game_id/draw-card", post(handlers::draw_card))
        .route(
            "/game/:game_id/draw-multiple",
            post(handlers::draw_multiple_cards),
        )
        .route("/game/:game_id/play-card", post(handlers::play_card))
        // Game state endpoints
        .route("/game/:game_id/game-state", get(handlers::get_game_state))
        .route("/game/:game_id/game-status", get(handlers::get_game_status))
        .route(
            "/game/:game_id/top-discard",
            get(handlers::get_top_discard_card),
        )
        // Utility endpoints
        .route("/game/:game_id/reset", post(handlers::reset_game))
        .route(
            "/game/:game_id/player/:player_id/hand-count",
            get(handlers::get_player_hand_count),
        )
        .route("/health", get(handlers::health_check))
        .with_state(games)
        .layer(cors);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("UNO backend server listening on 0.0.0.0:3001");

    axum::serve(listener, app).await?;
    Ok(())
}
