// backend/src/game/operations.rs

use super::{GameState, PlayerId};
use alloy::primitives::U256;
use anyhow::{anyhow, Result};
use zunnogame_lib::shuffle_deck;

/// Draw a single card for a player
pub fn draw_card(game_state: &mut GameState, player_id: PlayerId) -> Result<u8> {
    if !game_state.is_initialized() {
        return Err(anyhow!("Game not initialized"));
    }

    if !game_state.is_valid_player(player_id) {
        return Err(anyhow!("Invalid player ID"));
    }

    // Handle empty draw pile (reshuffle discard)
    if game_state.draw_pile.is_empty() {
        if game_state.discard_pile.len() <= 1 {
            return Err(anyhow!("No cards available"));
        }

        let top_card = game_state.discard_pile.pop().unwrap();
        game_state.draw_pile.append(&mut game_state.discard_pile);
        game_state.discard_pile.push(top_card);

        // Reshuffle with derived seed
        let new_seed = game_state.seed_metadata.value.wrapping_add(U256::from(1));
        shuffle_deck(&mut game_state.draw_pile, new_seed);
    }

    let card = game_state
        .draw_pile
        .pop()
        .ok_or_else(|| anyhow!("Draw pile empty"))?;

    game_state.player_hands[player_id as usize].push(card);
    Ok(card)
}

// Draw multiple cards (for Draw 2, Draw 4 penalties)
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
