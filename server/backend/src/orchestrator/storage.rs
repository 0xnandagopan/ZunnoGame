// backend/src/orchestrator/storage.rs

use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

/// Represents a game waiting for VRF fulfillment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingGame {
    pub session_id: String,
    pub vrf_request_id: U256,
    pub vrf_block_number: u64,
    pub num_players: u8,
    pub cards_per_player: u8,
    pub requested_at: u64,
    pub status: GameStatus,
}

/// Status of a game in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    /// Initial request received, requesting VRF
    Requesting,
    /// VRF request sent, waiting for fulfillment
    WaitingForVRF,
    /// VRF fulfilled, generating ZK proof
    GeneratingProof,
    /// Game ready to play
    Ready,
    /// Error occurred
    Failed(String),
}

/// Response for game initiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInitiation {
    pub session_id: String,
    pub status: GameStatus,
    pub estimated_wait_seconds: u64,
    pub vrf_request_id: U256,
}

/// Response for game status queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatusResponse {
    pub session_id: String,
    pub status: GameStatus,
    pub elapsed_seconds: u64,
    pub vrf_request_id: Option<U256>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionOutput {
    pub id: String,
    pub timestamp: String,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_cid: Option<String>,
}

/// Helper to get current Unix timestamp
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
