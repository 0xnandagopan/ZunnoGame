// backend/src/game/state.rs

use crate::blockchain::BlockchainSeed;
use serde::{Deserialize, Serialize};

pub type PlayerId = u8;

// UNO card deck mapping - matches JavaScript PACK_OF_CARDS exactly
pub const PACK_OF_CARDS: [&str; 108] = [
    "0R", "1R", "1R", "2R", "2R", "3R", "3R", "4R", "4R", "5R", "5R", "6R", "6R", "7R", "7R", "8R",
    "8R", "9R", "9R", "skipR", "skipR", "_R", "_R", "D2R", "D2R", "0G", "1G", "1G", "2G", "2G",
    "3G", "3G", "4G", "4G", "5G", "5G", "6G", "6G", "7G", "7G", "8G", "8G", "9G", "9G", "skipG",
    "skipG", "_G", "_G", "D2G", "D2G", "0B", "1B", "1B", "2B", "2B", "3B", "3B", "4B", "4B", "5B",
    "5B", "6B", "6B", "7B", "7B", "8B", "8B", "9B", "9B", "skipB", "skipB", "_B", "_B", "D2B",
    "D2B", "0Y", "1Y", "1Y", "2Y", "2Y", "3Y", "3Y", "4Y", "4Y", "5Y", "5Y", "6Y", "6Y", "7Y",
    "7Y", "8Y", "8Y", "9Y", "9Y", "skipY", "skipY", "_Y", "_Y", "D2Y", "D2Y", "W", "W", "W", "W",
    "D4W", "D4W", "D4W", "D4W",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player_hands: Vec<Vec<u8>>,
    pub draw_pile: Vec<u8>,
    pub discard_pile: Vec<u8>,
    pub is_shuffled: bool,
    pub seed_metadata: BlockchainSeed,
    pub proof_cid: Option<String>, // IPFS CID
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player_hands: Vec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            is_shuffled: false,
            seed_metadata: BlockchainSeed::default(),
            proof_cid: None,
        }
    }

    pub fn player_count(&self) -> usize {
        self.player_hands.len()
    }

    pub fn is_valid_player(&self, player_id: PlayerId) -> bool {
        (player_id as usize) < self.player_hands.len()
    }

    pub fn is_initialized(&self) -> bool {
        self.is_shuffled && !self.player_hands.is_empty()
    }

    /// Get total cards in circulation (for debugging)
    pub fn total_cards(&self) -> usize {
        let hands_total: usize = self.player_hands.iter().map(|hand| hand.len()).sum();
        hands_total + self.draw_pile.len() + self.discard_pile.len()
    }
}
