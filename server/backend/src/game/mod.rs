use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::blockchain;
use zunnogame_lib::{ShuffleOutcome, perform_shuffle};

// Consistent type - use u8 since MAX_PLAYERS = 10
pub type PlayerId = u8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player_hands: Vec<Vec<u8>>,
    pub draw_pile: Vec<u8>,
    pub discard_pile: Vec<u8>,
    pub is_shuffled: bool,
    pub seed_used: u64, // For debugging/replay
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player_hands: Vec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            is_shuffled: false,
            seed_used: 0,
        }
    }

    /// Get number of players in the game
    pub fn player_count(&self) -> usize {
        self.player_hands.len()
    }

    /// Check if player ID is valid
    pub fn is_valid_player(&self, player_id: PlayerId) -> bool {
        (player_id as usize) < self.player_hands.len()
    }

    /// Check if game is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.is_shuffled && !self.player_hands.is_empty()
    }

    /// Get total cards in circulation (for debugging)
    pub fn total_cards(&self) -> usize {
        let hands_total: usize = self.player_hands.iter().map(|hand| hand.len()).sum();
        hands_total + self.draw_pile.len() + self.discard_pile.len()
    }
}

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

/// Convert index to card string (matches JavaScript side)
pub fn index_to_card(index: u8) -> &'static str {
    PACK_OF_CARDS[index as usize]
}

pub async fn shuffle_and_deal(num_players: u8, cards_per_player: u8) -> Result<GameState> {

    // Get blockchain seed
    let seed = blockchain::get_random_seed()
        .await
        .map_err(|e| anyhow!("Failed to get blockchain seed: {}", e))?
        .to(); // Convert U256 to u64

    let result: ShuffleOutcome = perform_shuffle(num_players, cards_per_player, seed);

    Ok(GameState {
        player_hands: result.player_hands,
        draw_pile: result.draw_pile,
        discard_pile: Vec::new(),
        is_shuffled: true,
        seed_used: seed,
    })
}

/// Synchronous version for testing
pub fn shuffle_and_deal_sync(
    num_players: u8,
    cards_per_player: u8,
    seed: u64,
) -> Result<GameState> {

    let result: ShuffleOutcome = perform_shuffle(num_players, cards_per_player, seed);

    Ok(GameState {
        player_hands: result.player_hands,
        draw_pile: result.draw_pile,
        discard_pile: Vec::new(),
        is_shuffled: true,
        seed_used: seed,
    })
}

// Return reference for efficiency, add owned version when needed
pub fn get_initial_hands_ref(game_state: &GameState, player_id: PlayerId) -> Result<&[u8]> {
    if !game_state.is_initialized() {
        return Err(anyhow!("Game has not been initialized yet"));
    }

    let player_index = player_id as usize;
    game_state
        .player_hands
        .get(player_index)
        .map(|hand| hand.as_slice())
        .ok_or_else(|| {
            anyhow!(
                "Player {} not found (valid range: 0-{})",
                player_id,
                game_state.player_hands.len().saturating_sub(1)
            )
        })
}

// Properly return owned Vec when needed
pub fn get_initial_hands(game_state: &GameState, player_id: PlayerId) -> Result<Vec<u8>> {
    get_initial_hands_ref(game_state, player_id).map(|hand| hand.to_vec())
}

// Check correct pile and handle reshuffle
pub fn draw_card(game_state: &mut GameState, player_id: PlayerId) -> Result<u8> {
    if !game_state.is_initialized() {
        return Err(anyhow!("Game has not been initialized yet"));
    }

    // Validate player first
    if !game_state.is_valid_player(player_id) {
        return Err(anyhow!(
            "Player {} not found (valid range: 0-{})",
            player_id,
            game_state.player_hands.len().saturating_sub(1)
        ));
    }

    // Handle empty draw pile (UNO rules: reshuffle discard pile)
    if game_state.draw_pile.is_empty() {
        if game_state.discard_pile.len() <= 1 {
            return Err(anyhow!("No cards available to draw"));
        }

        // Keep top discard card, shuffle rest back to draw pile
        let top_card = game_state.discard_pile.pop().unwrap();
        game_state.draw_pile.append(&mut game_state.discard_pile);
        game_state.discard_pile.push(top_card);

        // Reshuffle draw pile with new seed
        let new_seed = game_state.seed_used.wrapping_add(1);
        shuffle_deck(&mut game_state.draw_pile, new_seed);
        game_state.seed_used = new_seed;
    }

    // Draw card
    let card = game_state
        .draw_pile
        .pop()
        .ok_or_else(|| anyhow!("Draw pile unexpectedly empty"))?;

    // Add to player's hand
    let player_index = player_id as usize;
    game_state.player_hands[player_index].push(card);

    Ok(card)
}

/// Draw multiple cards (for Draw 2, Draw 4 penalties)
pub fn draw_multiple_cards(
    game_state: &mut GameState,
    player_id: PlayerId,
    count: u8,
) -> Result<Vec<u8>> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut drawn_cards = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let card = draw_card(game_state, player_id)?;
        drawn_cards.push(card);
    }
    Ok(drawn_cards)
}

/// Play a card to the discard pile
pub fn play_card(game_state: &mut GameState, player_id: PlayerId, card_index: usize) -> Result<u8> {
    if !game_state.is_initialized() {
        return Err(anyhow!("Game has not been initialized yet"));
    }

    if !game_state.is_valid_player(player_id) {
        return Err(anyhow!("Player {} not found", player_id));
    }

    let player_index = player_id as usize;
    let hand = &mut game_state.player_hands[player_index];

    if card_index >= hand.len() {
        return Err(anyhow!(
            "Card index {} out of bounds (hand has {} cards)",
            card_index,
            hand.len()
        ));
    }

    let played_card = hand.remove(card_index);
    game_state.discard_pile.push(played_card);

    Ok(played_card)
}

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
    pub seed_used: u64,
}

impl From<&GameState> for GameStateJS {
    fn from(game_state: &GameState) -> Self {
        Self {
            player_hands: convert_distributed_cards_to_js(&game_state.player_hands),
            draw_pile: convert_indexes_to_js_cards(&game_state.draw_pile),
            discard_pile: convert_indexes_to_js_cards(&game_state.discard_pile),
            is_shuffled: game_state.is_shuffled,
            seed_used: game_state.seed_used,
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
