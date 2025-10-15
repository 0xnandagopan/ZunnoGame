//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use alloy_primitives::U256;
use alloy_sol_types::SolType;
use clap::Parser;
use sp1_sdk::{include_elf, HashableKey, ProverClient, SP1Stdin};
use zunnogame_lib::PublicValuesStruct;

use serde::{Deserialize, Serialize};
use sp1_zkv_sdk::*; // for the `convert_to_zkv` and `hash_bytes` methods.
use std::{fs::File, io::Write};

// Struct of the output we need
#[derive(Serialize, Deserialize)]
struct Output {
    image_id: String,
    pub_inputs: String,
    proof: String,
}

// Helper function to get hex strings
fn to_hex_with_prefix(bytes: &[u8]) -> String {
    let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}", hex_string)
}

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZUNNOGAME_ELF: &[u8] = include_elf!("zunno-program");

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    let p: u8 = 2;
    stdin.write(&p);

    let c: u8 = 5;
    stdin.write(&c);

    let r: u64 = U256::from(12345);
    stdin.write(&r);

    println!("inputs: p = {}, c = {}, r={}", p, c, r);

    // Setup the prover client.
    let client = ProverClient::from_env();

    let (_, report) = client.execute(ZUNNOGAME_ELF, &stdin).run().unwrap();
    println!(
        "executed program with {} cycles",
        report.total_instruction_count()
    );

    let (pk, vk) = client.setup(ZUNNOGAME_ELF);

    // Generate the proof
    let proof = client
        .prove(&pk, &stdin)
        .compressed()
        .run()
        .expect("failed to generate proof");

    println!("Successfully generated proof!");

    // Convert proof and vk into a zkVerify-compatible proof.
    let SP1ZkvProofWithPublicValues {
        proof: shrunk_proof,
        public_values,
    } = client
        .convert_proof_to_zkv(proof, Default::default())
        .unwrap();
    let vk_hash = vk.hash_bytes();

    // Serialize the proof
    let serialized_proof = bincode::serde::encode_to_vec(&shrunk_proof, bincode::config::legacy())
        .expect("failed to serialize proof");

    // Convert to required struct
    let output = Output {
        proof: to_hex_with_prefix(&serialized_proof),
        image_id: to_hex_with_prefix(&vk_hash),
        pub_inputs: to_hex_with_prefix(&public_values),
    };

    // Convert to JSON and store in the file
    let json_string = serde_json::to_string_pretty(&output).expect("Failed to serialize to JSON.");

    let mut file = File::create("proof.json").unwrap();
    file.write_all(json_string.as_bytes()).unwrap();
}
