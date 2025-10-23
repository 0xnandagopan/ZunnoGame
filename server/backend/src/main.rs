// backend/src/main.rs

mod api;
mod blockchain;
mod game;
mod orchestrator;
mod proof_management;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use rustls::crypto::{ring, CryptoProvider};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<()> {
    CryptoProvider::install_default(ring::default_provider())
        .expect("Failed to install default crypto provider");

    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("backend=debug".parse().unwrap())
                .add_directive("tower_http=debug".parse().unwrap())
                .add_directive("zunnogame_script=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting Zunno Game Server");

    // Initialize blockchain adapter
    tracing::info!("Initializing blockchain adapter...");
    let blockchain = blockchain::BlockchainAdapter::new().await?;
    tracing::info!("Blockchain adapter initialized");

    // Initialize orchestrator
    tracing::info!("Initializing game orchestrator...");
    let orchestrator = Arc::new(orchestrator::GameOrchestrator::new(blockchain).await?);
    tracing::info!("Game orchestrator initialized");

    // Start background tasks
    tracing::info!("Starting background tasks...");
    orchestrator.clone().start_background_tasks();
    tracing::info!("Background tasks started");

    // Build API routes
    let app = Router::new()
        .route("/api/game/start", post(api::start_game))
        .route("/api/game/:session_id/status", get(api::get_game_status))
        .route("/api/game/:session_id", get(api::get_game_state))
        .route("/api/game/:session_id/proof", get(api::get_game_proof))
        .route("/health", get(|| async { "OK" }))
        .layer(TraceLayer::new_for_http())
        .with_state(orchestrator);

    // Start server
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Available endpoints:");
    tracing::info!("  POST   /api/game/start");
    tracing::info!("  GET    /api/game/:session_id/status");
    tracing::info!("  GET    /api/game/:session_id");
    tracing::info!("  GET    /api/game/:session_id/proof");
    tracing::info!("  GET    /health");

    axum::serve(listener, app).await?;

    Ok(())
}
