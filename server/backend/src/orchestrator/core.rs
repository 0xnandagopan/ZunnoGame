// backend/src/orchestrator/core.rs

use alloy::primitives::U256;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::storage::{
    current_timestamp, u256_to_bytes32, ActionOutput, GameInitiation, GameStatus,
    GameStatusResponse, PendingGame,
};
use crate::blockchain::{BlockchainAdapter, BlockchainSeed};
use crate::game::{perform_shuffle, GameState};
use crate::proof_management::{
    config::IpfsProvider,
    retry_service::{IpfsService, IpfsUploadConfig},
};
use zunnogame_script::{ProofGenerator, ProofInput, ProofOutput};

/// Main orchestrator that coordinates VRF requests, game initialization, and state management
#[derive(Clone)]
pub struct GameOrchestrator {
    /// In-memory storage of pending games
    pending_games: Arc<RwLock<HashMap<String, PendingGame>>>,
    /// Completed games ready to play
    completed_games: Arc<RwLock<HashMap<String, GameState>>>,
    /// Blockchain adapter for VRF operations
    blockchain: Arc<BlockchainAdapter>,
    // Proof generator (expensive to create, reuse)
    proof_generator: Arc<ProofGenerator>,
}

impl GameOrchestrator {
    /// Create a new game orchestrator
    pub async fn new(blockchain: BlockchainAdapter) -> Result<Self> {
        tracing::info!("Initializing proof generator...");
        let proof_generator = Arc::new(ProofGenerator::new()?);
        tracing::info!("Proof generator ready");

        Ok(Self {
            pending_games: Arc::new(RwLock::new(HashMap::new())),
            completed_games: Arc::new(RwLock::new(HashMap::new())),
            blockchain: Arc::new(blockchain),
            proof_generator,
        })
    }

    /// Start background tasks (VRF listener, cleanup)
    pub fn start_background_tasks(self: Arc<Self>) {
        // Spawn VRF fulfillment checker
        let orchestrator = self.clone();
        tokio::spawn(async move {
            orchestrator.run_vrf_fulfillment_loop().await;
        });

        // Spawn cleanup task for expired games
        let orchestrator = self.clone();
        tokio::spawn(async move {
            orchestrator.cleanup_expired_games().await;
        });
    }

    /// Initiate a new game
    ///
    /// Returns immediately with session ID while VRF request is processed in background
    pub async fn initiate_game(
        &self,
        num_players: u8,
        cards_per_player: u8,
    ) -> Result<GameInitiation> {
        tracing::info!(
            num_players = num_players,
            cards_per_player = cards_per_player,
            "Initiating new game"
        );

        // Generate unique session ID
        let session_id = Uuid::new_v4().to_string();

        // Create pending game entry
        let pending = PendingGame {
            session_id: session_id.clone(),
            vrf_request_id: U256::ZERO,
            vrf_block_number: 0,
            num_players,
            cards_per_player,
            requested_at: current_timestamp(),
            status: GameStatus::Requesting,
        };

        // Store pending game
        self.pending_games
            .write()
            .await
            .insert(session_id.clone(), pending);

        // Request VRF in background (non-blocking)
        let orchestrator = self.clone();
        let session_id_clone = session_id.clone();

        tokio::spawn(async move {
            if let Err(e) = orchestrator.request_vrf_for_game(&session_id_clone).await {
                tracing::error!(
                    session_id = %session_id_clone,
                    error = %e,
                    "Failed to request VRF"
                );

                // Update status to failed
                let mut games = orchestrator.pending_games.write().await;
                if let Some(game) = games.get_mut(&session_id_clone) {
                    game.status = GameStatus::Failed(e.to_string());
                }
            }
        });

        // Return immediately with session info
        Ok(GameInitiation {
            session_id,
            status: GameStatus::Requesting,
            estimated_wait_seconds: 60, // Estimate for VRF + proof generation
            vrf_request_id: U256::ZERO, // Will be updated after VRF request
        })
    }

    /// Get current status of a game
    pub async fn get_game_status(&self, session_id: &str) -> Result<GameStatusResponse> {
        // Check if game is completed
        if self.completed_games.read().await.contains_key(session_id) {
            return Ok(GameStatusResponse {
                session_id: session_id.to_string(),
                status: GameStatus::Ready,
                elapsed_seconds: 0, // Game is ready
                vrf_request_id: None,
            });
        }

        // Check pending games
        let games = self.pending_games.read().await;
        if let Some(pending) = games.get(session_id) {
            let elapsed = current_timestamp() - pending.requested_at;
            return Ok(GameStatusResponse {
                session_id: session_id.to_string(),
                status: pending.status.clone(),
                elapsed_seconds: elapsed,
                vrf_request_id: Some(pending.vrf_request_id),
            });
        }

        Err(anyhow!("Game session not found: {}", session_id))
    }

    /// Get completed game state
    pub async fn get_game_state(&self, session_id: &str) -> Result<GameState> {
        self.completed_games
            .read()
            .await
            .get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("Game not ready or not found: {}", session_id))
    }

    /// Request VRF for a specific game session
    async fn request_vrf_for_game(&self, session_id: &str) -> Result<()> {
        tracing::info!(session_id = session_id, "Requesting VRF for game");

        // Request VRF from blockchain
        let vrf_request = self.blockchain.request_vrf().await?;

        tracing::info!(
            session_id = session_id,
            request_id = %vrf_request.request_id,
            block_number = vrf_request.block_number,
            "VRF request successful"
        );

        // Update pending game with VRF details
        let mut games = self.pending_games.write().await;
        if let Some(game) = games.get_mut(session_id) {
            game.vrf_request_id = vrf_request.request_id;
            game.vrf_block_number = vrf_request.block_number;
            game.status = GameStatus::WaitingForVRF;
        }

        Ok(())
    }

    /// Background loop that checks for VRF fulfillment
    async fn run_vrf_fulfillment_loop(&self) {
        tracing::info!("Starting VRF fulfillment checker loop");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // Get all pending games waiting for VRF
            let pending_games = self.pending_games.read().await;
            let games_to_check: Vec<_> = pending_games
                .values()
                .filter(|g| g.status == GameStatus::WaitingForVRF)
                .cloned()
                .collect();
            drop(pending_games);

            // Check each game for VRF fulfillment
            for pending_game in games_to_check {
                let orchestrator = self.clone();
                let session_id = pending_game.session_id.clone();
                let request_id = pending_game.vrf_request_id;
                let block_number = pending_game.vrf_block_number;
                let num_players = pending_game.num_players;
                let cards_per_player = pending_game.cards_per_player;

                tokio::spawn(async move {
                    if let Err(e) = orchestrator
                        .check_and_finalize_game(
                            &session_id,
                            request_id,
                            block_number,
                            num_players,
                            cards_per_player,
                        )
                        .await
                    {
                        tracing::debug!(
                            session_id = %session_id,
                            error = %e,
                            "VRF not yet fulfilled or error occurred"
                        );
                    }
                });
            }
        }
    }

    /// Check if VRF is fulfilled and finalize the game
    async fn check_and_finalize_game(
        &self,
        session_id: &str,
        request_id: U256,
        block_number: u64,
        num_players: u8,
        cards_per_player: u8,
    ) -> Result<()> {
        tracing::debug!(
            session_id = session_id,
            request_id = %request_id,
            "Checking VRF fulfillment"
        );

        // Try to get random value (with short timeout for polling approach)
        let random_value = self
            .blockchain
            .get_randomness(request_id, block_number, 10)
            .await?;

        tracing::info!(
            session_id = session_id,
            request_id = %request_id,
            random_value = %random_value,
            "VRF fulfilled! Finalizing game..."
        );

        // Update status to generating proof
        {
            let mut games = self.pending_games.write().await;
            if let Some(game) = games.get_mut(session_id) {
                game.status = GameStatus::GeneratingProof;
            }
        }

        // Finalize the game
        self.finalize_game(
            session_id,
            random_value,
            request_id,
            num_players,
            cards_per_player,
        )
        .await?;

        Ok(())
    }

    /// Finalize game: shuffle, generate proof, store state
    async fn finalize_game(
        &self,
        session_id: &str,
        random_value: U256,
        request_id: U256,
        num_players: u8,
        cards_per_player: u8,
    ) -> Result<()> {
        tracing::info!(session_id = session_id, "Finalizing game with VRF seed");

        let seed_bytes = u256_to_bytes32(random_value);

        // Perform shuffle
        let shuffle_outcome = perform_shuffle(num_players, cards_per_player, seed_bytes)?;

        tracing::info!(session_id = session_id, "Shuffle complete");

        tracing::info!(session_id = session_id, "Generating ZK proof...");

        let proof_result = tokio::task::spawn_blocking({
            let proof_generator = self.proof_generator.clone();

            move || {
                proof_generator.generate_proof(ProofInput {
                    num_players,
                    cards_per_player,
                    seed: seed_bytes,
                })
            }
        })
        .await
        .map_err(|e| anyhow!("Proof generation task panicked: {}", e))??;

        tracing::info!(
            session_id = session_id,
            proof_id = %proof_result.image_id[..18],
            "Proof generated successfully"
        );

        let output = match serde_json::to_string_pretty(&proof_result) {
            Ok(json_data) => ActionOutput {
                id: session_id.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                data: json_data, // String representation of the proof result
                ipfs_cid: None,
            },
            Err(e) => {
                // Handle the error appropriately
                tracing::error!("Failed to serialize proof_result: {}", e);
                return Err(anyhow!("Proof output serialization failed: {}", err));
            }
        };

        let proof_cid: String = self.upload_proof(output).await?;

        tracing::info!(
            session_id = session_id.to_string(),
            proof_cid = %proof_cid,
            "Proof stored"
        );

        // Create game state
        let game_state = GameState {
            player_hands: shuffle_outcome.player_hands,
            draw_pile: shuffle_outcome.draw_pile,
            discard_pile: Vec::new(),
            is_shuffled: true,
            seed_metadata: BlockchainSeed {
                value: random_value,
                request_id,
            },
            proof_cid,
        };

        // Store completed game
        self.completed_games
            .write()
            .await
            .insert(session_id.to_string(), game_state);

        // Update pending game status
        let mut games = self.pending_games.write().await;
        if let Some(game) = games.get_mut(session_id) {
            game.status = GameStatus::Ready;
        }

        tracing::info!(session_id = session_id, "Game ready!");

        Ok(())
    }

    async fn upload_proof(&self, output: ActionOutput) -> Result<String> {
        // Initialize IPFS service
        let provider = IpfsProvider::from_env()?;
        let config = IpfsUploadConfig::default();
        let ipfs_service = IpfsService::new(provider, config);

        // Upload to IPFS
        match ipfs_service.upload_with_retry(&output).await {
            Ok(cid) => {
                return Ok(cid);
            }
            Err(err) => {
                tracing::error!("Failed to upload proof to IPFS: {}", err);
                return Err(anyhow!("Proof upload failed: {}", err));
            }
        }
    }

    /// Cleanup expired pending games (older than 10 minutes)
    async fn cleanup_expired_games(&self) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

            let mut games = self.pending_games.write().await;
            let now = current_timestamp();

            games.retain(|session_id, game| {
                let age = now - game.requested_at;
                let keep = age < 600; // 10 minutes

                if !keep {
                    tracing::info!(
                        session_id = session_id,
                        age_seconds = age,
                        "Cleaning up expired game"
                    );
                }

                keep
            });
        }
    }
}
