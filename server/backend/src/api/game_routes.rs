// backend/src/api/game_routes.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::game::GameStateJS;
use crate::orchestrator::{GameInitiation, GameOrchestrator, GameStatusResponse};
use zunnogame_script::ProofOutput;

/// Request body for starting a new game
#[derive(Debug, Deserialize)]
pub struct StartGameRequest {
    pub num_players: u8,
    pub cards_per_player: u8,
}

/// Response for get game state
#[derive(Debug, Serialize)]
pub struct GameStateApiResponse {
    pub session_id: String,
    pub game_state: GameStateJS,
}

/// Response for proof retrieval
#[derive(Debug, Serialize)]
pub struct ProofResponse {
    pub session_id: String,
    pub proof_cid: String,
}

/// POST /api/game/start - Initiate a new game
pub async fn start_game(
    State(orchestrator): State<Arc<GameOrchestrator>>,
    Json(req): Json<StartGameRequest>,
) -> Result<Json<GameInitiation>, (StatusCode, String)> {
    tracing::info!(
        num_players = req.num_players,
        cards_per_player = req.cards_per_player,
        "API: Start game request received"
    );

    match orchestrator
        .initiate_game(req.num_players, req.cards_per_player)
        .await
    {
        Ok(initiation) => {
            tracing::info!(
                session_id = %initiation.session_id,
                "API: Game initiated successfully"
            );
            Ok(Json(initiation))
        }
        Err(e) => {
            tracing::error!(error = %e, "API: Failed to initiate game");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to start game: {}", e),
            ))
        }
    }
}

/// GET /api/game/:session_id/status - Get game status
pub async fn get_game_status(
    State(orchestrator): State<Arc<GameOrchestrator>>,
    Path(session_id): Path<String>,
) -> Result<Json<GameStatusResponse>, (StatusCode, String)> {
    tracing::debug!(session_id = %session_id, "API: Get game status");

    match orchestrator.get_game_status(&session_id).await {
        Ok(status) => Ok(Json(status)),
        Err(e) => {
            tracing::warn!(
                session_id = %session_id,
                error = %e,
                "API: Game not found"
            );
            Err((StatusCode::NOT_FOUND, format!("Game not found: {}", e)))
        }
    }
}

/// GET /api/game/:session_id - Get complete game state
pub async fn get_game_state(
    State(orchestrator): State<Arc<GameOrchestrator>>,
    Path(session_id): Path<String>,
) -> Result<Json<GameStateApiResponse>, (StatusCode, String)> {
    tracing::debug!(session_id = %session_id, "API: Get game state");

    match orchestrator.get_game_state(&session_id).await {
        Ok(game_state) => {
            let game_state_js = GameStateJS::from(&game_state);
            Ok(Json(GameStateApiResponse {
                session_id,
                game_state: game_state_js,
            }))
        }
        Err(e) => {
            tracing::warn!(
                session_id = %session_id,
                error = %e,
                "API: Game not ready or not found"
            );
            Err((StatusCode::NOT_FOUND, format!("Game not ready: {}", e)))
        }
    }
}

/// GET /api/game/:session_id/proof - Get ZK proof for game
pub async fn get_game_proof(
    State(orchestrator): State<Arc<GameOrchestrator>>,
    Path(session_id): Path<String>,
) -> Result<Json<ProofResponse>, (StatusCode, String)> {
    tracing::debug!(session_id = %session_id, "API: Get game proof");

    // Get game state to verify it's ready
    match orchestrator.get_game_state(&session_id).await {
        Ok(game_state) => {
            let proof_cid = game_state.proof_cid.clone();
            let proof_response = ProofResponse {
                session_id,
                proof_cid,
            };
            Ok(Json(proof_response))
        }
        Err(e) => {
            tracing::warn!(
                session_id = %session_id,
                error = %e,
                "API: Game not found"
            );
            Err((StatusCode::NOT_FOUND, format!("Game not found: {}", e)))
        }
    }
}

// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "uno-game-api",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}
