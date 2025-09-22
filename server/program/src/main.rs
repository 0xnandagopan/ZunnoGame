//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use sha2::{Digest, Sha256};
use zunnogame_lib::PublicValuesStruct;
use zunnogame_lib::{perform_shuffle, ShuffleOutcome};

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let p = sp1_zkvm::io::read::<u8>();
    let c = sp1_zkvm::io::read::<u8>();
    let r = sp1_zkvm::io::read::<u64>();

    // perform shuffle and deal and accept the result.
    match perform_shuffle(p, c, r) {
        Ok(outcome) => {
            let initial_player_hands = outcome.player_hands;
            let mut player_card_hashes = Vec::new();
            for player_cards in &initial_player_hands {
                let mut hasher = Sha256::new();
                hasher.update(player_cards);
                let player_hash = hasher.finalize().to_vec();
                player_card_hashes.push(player_hash.into());
            }
            let seed_used = r;

            let publc_values = PublicValuesStruct {
                no_of_players: p,
                cards_per_player: c,
                initial_hands_hash: player_card_hashes,
                seed: seed_used,
            };

            // Encode the public values of the program.
            let bytes = PublicValuesStruct::abi_encode(&publc_values);

            // Commit to the public values of the program. The final proof will have a commitment to all the
            // bytes that were committed to.
            sp1_zkvm::io::commit_slice(&bytes);
        }
        Err(err) => {
            panic!("Invalid game parameters or shuffle failed");
        }
    }
}
