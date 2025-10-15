// script/src/bin/main.rs (REFACTORED)

use alloy_primitives::U256;
use anyhow::Result;
use clap::Parser;
use std::{fs::File, io::Write};

// Import from library
use zunnogame_script::{ProofGenerator, ProofInput, ProofOutput};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of players
    #[arg(short, long, default_value_t = 2)]
    players: u8,

    /// Cards per player
    #[arg(short, long, default_value_t = 7)]
    cards: u8,

    /// Seed value (as decimal)
    #[arg(short, long, default_value_t = 12345)]
    seed: u64,

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
    println!("Seed: {}", args.seed);
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
        seed: U256::from(args.seed),
    };

    // Generate proof
    println!("Generating proof...");
    let output = generator.generate_proof(input)?;
    println!("✓ Proof generated");
    println!();

    // Write to file
    println!("Writing to {}...", args.output);
    let json_string: Result<String> = serde_json::to_string_pretty(&output);
    let mut file = File::create(&args.output)?;
    file.write_all(json_string.as_bytes())?;
    println!("✓ Written");
    println!();

    println!("=== Success ===");
    println!("Proof ID: {}", &output.image_id[..18]);
    println!("Public inputs: {}", &output.pub_inputs[..18]);

    Ok(())
}
