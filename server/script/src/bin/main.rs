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

use alloy_sol_types::SolType;
use clap::Parser;
use sha2::{Digest, Sha256};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use zunnogame_lib::PublicValuesStruct;

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

    let r: u64 = 1234567890;
    stdin.write(&r);

    println!("inputs: p = {}, c = {}, r={}", p, c, r);

    // Setup the prover client.
    let client = ProverClient::from_env();

    // let (_, report) = client.execute(ZUNNOGAME_ELF, &stdin).run().unwrap();
    // println!(
    //     "executed program with {} cycles",
    //     report.total_instruction_count()
    // );

    let (pk, vk) = client.setup(ZUNNOGAME_ELF);

    // Generate the proof
    let proof = client
        .prove(&pk, &stdin)
        .compressed()
        .run()
        .expect("failed to generate proof");

    println!("Successfully generated proof!");
}
