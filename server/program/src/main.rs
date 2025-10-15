// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy::primitives::U256;
use alloy_sol_types::SolType;
use sha2::{Digest, Sha256};
use zunnogame_lib::{perform_shuffle, u256_to_bytes32, PublicValuesStruct};

pub fn main() {
    // Read inputs
    let p = sp1_zkvm::io::read::<u8>(); // players
    let c = sp1_zkvm::io::read::<u8>(); // cards per player
    let r = sp1_zkvm::io::read::<U256>(); // 256-bit seed

    match perform_shuffle(p, c, r) {
        Ok(outcome) => {
            // ========================================
            // Proof: Prove shuffle is valid permutation
            // ========================================
            let original_deck: Vec<u8> = (0..108).collect();

            // Reconstruct full shuffled deck
            let mut shuffled_deck = Vec::new();
            for hand in &outcome.player_hands {
                shuffled_deck.extend_from_slice(hand);
            }
            shuffled_deck.extend_from_slice(&outcome.draw_pile);

            // Verify it's a permutation
            assert_eq!(shuffled_deck.len(), 108, "Deck must have exactly 108 cards");

            let mut sorted_original = original_deck.clone();
            let mut sorted_shuffled = shuffled_deck.clone();
            sorted_original.sort_unstable();
            sorted_shuffled.sort_unstable();

            assert_eq!(
                sorted_original, sorted_shuffled,
                "Shuffle must be a valid permutation - no duplicates or missing cards"
            );

            let seed_value: [u8; 32] = u256_to_bytes32(r);

            // ========================================
            // Proof: Build Merkle tree for card proofs
            // ========================================
            let card_leaves: Vec<[u8; 32]> = shuffled_deck
                .iter()
                .enumerate()
                .map(|(position, &card_value)| {
                    let mut hasher = Sha256::new();
                    hasher.update(b"ZUNNO_CARD_LEAF_V1");
                    hasher.update(&seed_value); // Bind to game seed
                    hasher.update(&(position as u64).to_le_bytes());
                    hasher.update(&[card_value]);
                    hasher.finalize().into()
                })
                .collect();

            let merkle_root = build_merkle_root(&card_leaves);

            // ========================================
            // Proof: Hash draw pile with commitment
            // ========================================
            let draw_pile_hash: [u8; 32] = {
                let mut hasher = Sha256::new();
                hasher.update(b"ZUNNO_DRAW_PILE_V1");
                hasher.update(&seed_value); // Bind to seed
                hasher.update(&outcome.draw_pile);
                hasher.finalize().into()
            };

            // ========================================
            // Proof: Hash player hands WITH SALT
            // ========================================
            let mut player_hand_hashes = Vec::new();
            for (player_id, player_cards) in outcome.player_hands.iter().enumerate() {
                // Derive unique salt per player
                let salt = {
                    let mut hasher = Sha256::new();
                    hasher.update(b"ZUNNO_PLAYER_SALT_V1");
                    hasher.update(&seed_value);
                    hasher.update(&[player_id as u8]);
                    hasher.finalize()
                };

                // Hash with salt
                let mut hasher = Sha256::new();
                hasher.update(&salt);
                hasher.update(player_cards);
                let player_hash = hasher.finalize().to_vec();
                player_hand_hashes.push(player_hash.into());
            }

            // ========================================
            // Commit to comprehensive public values
            // ========================================
            let public_values = PublicValuesStruct {
                no_of_players: p,
                cards_per_player: c,
                initial_hands_hash: player_hand_hashes,
                draw_pile_hash: draw_pile_hash.into(),
                merkle_root: merkle_root.into(),
                seed: seed_value.into(),
            };

            let bytes = PublicValuesStruct::abi_encode(&public_values);
            sp1_zkvm::io::commit_slice(&bytes);
        }
        Err(_) => {
            panic!("Invalid game parameters or shuffle failed");
        }
    }
}

// Helper function
fn build_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut current_layer = leaves.to_vec();

    while current_layer.len() > 1 {
        let mut next_layer = Vec::new();

        for chunk in current_layer.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() == 2 { chunk[1] } else { chunk[0] };

            let mut hasher = Sha256::new();
            hasher.update(b"ZUNNO_MERKLE_NODE_V1");
            if left <= right {
                hasher.update(&left);
                hasher.update(&right);
            } else {
                hasher.update(&right);
                hasher.update(&left);
            }

            next_layer.push(hasher.finalize().into());
        }

        current_layer = next_layer;
    }

    current_layer[0]
}
