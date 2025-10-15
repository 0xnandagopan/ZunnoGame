// backend/src/game/mod.rs

mod operations;
mod state;

pub use operations::{
    draw_card, draw_multiple_cards, get_initial_hands, get_initial_hands_ref, play_card,
};
pub use state::{GameState, PlayerId, PACK_OF_CARDS};

// Re-export from lib for convenience
pub use zunnogame_lib::{
    perform_shuffle, validate_game_params, ShuffleOutcome, DECK_SIZE, MAX_CARDS_PER_PLAYER,
    MAX_PLAYERS,
};

use alloy::primitives::U256;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Convert index to card string (matches JavaScript side)
pub fn index_to_card(index: u8) -> &'static str {
    PACK_OF_CARDS[index as usize]
}

// pub async fn shuffle_and_deal(num_players: u8, cards_per_player: u8) -> Result<GameState> {
//     // Get blockchain seed
//     let blockchain_seed = blockchain::get_random_seed().await?;

//     let result: ShuffleOutcome =
//         perform_shuffle(num_players, cards_per_player, blockchain_seed.value)?;

//     Ok(GameState {
//         player_hands: result.player_hands,
//         draw_pile: result.draw_pile,
//         discard_pile: Vec::new(),
//         is_shuffled: true,
//         seed_used: blockchain_seed.value,
//     })
// }

/// Synchronous version for testing
// pub fn shuffle_and_deal_sync(
//     num_players: u8,
//     cards_per_player: u8,
//     seed: U256,
// ) -> Result<GameState> {
//     let result: ShuffleOutcome = perform_shuffle(num_players, cards_per_player, seed)?;

//     Ok(GameState {
//         player_hands: result.player_hands,
//         draw_pile: result.draw_pile,
//         discard_pile: Vec::new(),
//         is_shuffled: true,
//         seed_used: seed,
//     })
// }

// ============================================================================
// JAVASCRIPT CONVERSION UTILITIES
// ============================================================================

/// Convert card indexes to JavaScript-compatible format
/// This function maps our u8 indexes to the exact string format your JavaScript expects
pub fn convert_indexes_to_js_cards(card_indexes: &[u8]) -> Vec<String> {
    card_indexes
        .iter()
        .map(|&index| PACK_OF_CARDS[index as usize].to_string())
        .collect()
}

/// Convert distributed card indexes to JavaScript format
/// Returns a nested structure matching your JavaScript game expectations
pub fn convert_distributed_cards_to_js(distributed_cards: &[Vec<u8>]) -> Vec<Vec<String>> {
    distributed_cards
        .iter()
        .map(|player_cards| convert_indexes_to_js_cards(player_cards))
        .collect()
}

/// Convert entire game state to JavaScript-compatible format
#[derive(Debug, Serialize, Deserialize)]
pub struct GameStateJS {
    pub player_hands: Vec<Vec<String>>,
    pub draw_pile: Vec<String>,
    pub discard_pile: Vec<String>,
    pub is_shuffled: bool,
    pub seed_used: U256,
}

impl From<&GameState> for GameStateJS {
    fn from(game_state: &GameState) -> Self {
        Self {
            player_hands: convert_distributed_cards_to_js(&game_state.player_hands),
            draw_pile: convert_indexes_to_js_cards(&game_state.draw_pile),
            discard_pile: convert_indexes_to_js_cards(&game_state.discard_pile),
            is_shuffled: game_state.is_shuffled,
            seed_used: game_state.seed_metadata.value,
        }
    }
}

/// Convert single card index to JavaScript format
pub fn convert_card_to_js(card_index: u8) -> String {
    PACK_OF_CARDS[card_index as usize].to_string()
}

/// Optimized conversion for API responses (avoids intermediate allocations)
pub fn serialize_player_hand_js(
    game_state: &GameState,
    player_id: PlayerId,
) -> Result<Vec<String>> {
    let hand = get_initial_hands_ref(game_state, player_id)?;
    Ok(convert_indexes_to_js_cards(hand))
}
