use alloy_primitives;
use alloy_primitives::U256;
use alloy_sol_types::sol;
use anyhow::{anyhow, Result};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

sol! {
    struct PublicValuesStruct {
        uint8 no_of_players;
        uint8 cards_per_player;
        bytes[] initial_hands_hash;
        bytes32 draw_pile_hash;
        bytes32 merkle_root;
        bytes32 seed;
    }
}

pub const DECK_SIZE: usize = 108;
pub const MAX_PLAYERS: u8 = 10;
pub const MAX_CARDS_PER_PLAYER: u8 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShuffleOutcome {
    pub player_hands: Vec<Vec<u8>>,
    pub draw_pile: Vec<u8>,
    pub draw_pile_count: u64,
}

pub fn u256_to_bytes32(value: U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&value.to_be_bytes::<32>());
    bytes
}

/// Fisher-Yates shuffle
pub fn shuffle_deck(deck: &mut [u8], seed: U256) {
    // Convert U256 seed to [u8; 32]
    let seed_bytes = u256_to_bytes32(seed);

    let mut rng = ChaCha20Rng::from_seed(seed_bytes);
    deck.shuffle(&mut rng);
}

/// Efficient card distribution using iterators
pub fn distribute_cards(deck: &[u8], num_players: u8, cards_per_player: u8) -> Vec<Vec<u8>> {
    let mut player_hands =
        vec![Vec::with_capacity(cards_per_player as usize); num_players as usize];

    deck.iter()
        .take((num_players as usize) * (cards_per_player as usize))
        .enumerate()
        .for_each(|(i, &card)| {
            let player_index = i % (num_players as usize);
            player_hands[player_index].push(card);
        });

    player_hands
}

/// Validate game parameters
pub fn validate_game_params(num_players: u8, cards_per_player: u8) -> Result<()> {
    if num_players == 0 || num_players > MAX_PLAYERS {
        return Err(anyhow!(
            "Invalid number of players: {} (must be 1-{})",
            num_players,
            MAX_PLAYERS
        ));
    }

    if cards_per_player == 0 || cards_per_player > MAX_CARDS_PER_PLAYER {
        return Err(anyhow!(
            "Invalid cards per player: {} (must be 1-{})",
            cards_per_player,
            MAX_CARDS_PER_PLAYER
        ));
    }

    let total_cards_needed = (num_players as usize) * (cards_per_player as usize);
    if total_cards_needed >= DECK_SIZE {
        return Err(anyhow!(
            "Not enough cards: need {} for {} players with {} cards each (deck has {})",
            total_cards_needed,
            num_players,
            cards_per_player,
            DECK_SIZE
        ));
    }

    Ok(())
}

pub fn perform_shuffle(
    num_players: u8,
    cards_per_player: u8,
    seed: U256,
) -> Result<ShuffleOutcome> {
    match validate_game_params(num_players, cards_per_player) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    // Create deck
    let mut deck: Vec<u8> = (0..DECK_SIZE as u8).collect();

    //shuffle deck
    shuffle_deck(&mut deck, seed);

    // Distribute cards
    let player_hands = distribute_cards(&deck, num_players, cards_per_player);

    // Create draw pile from remaining cards
    let total_cards_needed = (num_players as usize) * (cards_per_player as usize);
    let draw_pile = deck[total_cards_needed..].to_vec();
    let count = draw_pile.len();
    let draw_pile_count: u64 = count.try_into().unwrap();

    // Create shuffle outcome
    let outcome = ShuffleOutcome {
        player_hands,
        draw_pile,
        draw_pile_count,
    };

    Ok(outcome)
}
