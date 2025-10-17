// script/src/bin/main.rs

use anyhow::{anyhow, Result};
use clap::Parser;
use std::{fs::File, io::Write};

// Import from library
use zunnogame_script::{ProofGenerator, ProofInput};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of players
    #[arg(short, long)]
    players: u8,

    /// Cards per player
    #[arg(short, long)]
    cards: u8,

    /// Seed value
    #[arg(short, long)]
    seed: [u8; 32],

    /// Output file path
    #[arg(short, long, default_value = "proof.json")]
    output: String,
}

fn main() -> Result<()> {
    // Setup logger
    sp1_sdk::utils::setup_logger();
    dotenvy::dotenv().ok();

    // Parse command-line arguments
    let args = Args::parse();

    println!("=== Zunno Game Proof Generator ===");
    println!("Players: {}", args.players);
    println!("Cards per player: {}", args.cards);
    println!("Output: {}", args.output);
    println!();

    // Initialize proof generator
    println!("Initializing proof generator...");
    let generator = ProofGenerator::new()?;
    println!("✓ Initialized");
    println!();

    // Prepare input
    let input = ProofInput {
        num_players: args.players,
        cards_per_player: args.cards,
        seed: args.seed,
    };

    // Generate proof
    println!("Generating proof...");
    let output = generator.generate_proof(input)?;
    println!("✓ Proof generated");
    println!();

    match serde_json::to_string_pretty(&output) {
        Ok(data) => {
            println!("Writing to {}...", args.output);
            let mut file = File::create(&args.output)?;
            file.write_all(data.as_bytes())?;
            println!("✓ Written");
            println!();

            println!("=== Success ===");
            println!("Proof ID: {}", &data.image_id[..18]);
            println!("Public inputs: {}", &data.pub_inputs[..18]);

            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to serialize proof output: {}", e);
            return Err(anyhow!("Proof output serialization failed: {}", err));
        }
    }
}
