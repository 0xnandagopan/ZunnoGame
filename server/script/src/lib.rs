// script/src/lib.rs
//
use alloy_sol_types::SolType;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, EnvProver, HashableKey, ProverClient, SP1Stdin};
use sp1_zkv_sdk::{SP1ZkvProofWithPublicValues, ZkvProver};
use zunnogame_lib::PublicValuesStruct;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZUNNOGAME_ELF: &[u8] = include_elf!("zunno-program");

/// Input parameters for proof generation
#[derive(Debug, Clone)]
pub struct ProofInput {
    pub num_players: u8,
    pub cards_per_player: u8,
    pub seed: [u8; 32],
}

/// Generated proof output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOutput {
    /// Hex-encoded proof data
    pub proof: String,
    /// Hex-encoded image ID (verification key hash)
    pub image_id: String,
    /// Hex-encoded public inputs
    pub pub_inputs: String,
}

/// SP1 Proof Generator
pub struct ProofGenerator {
    client: EnvProver,
    pk: sp1_sdk::SP1ProvingKey,
    vk: sp1_sdk::SP1VerifyingKey,
}

impl ProofGenerator {
    /// Initialize the proof generator (expensive - do once)
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing SP1 proof generator...");

        let client = ProverClient::from_env();

        tracing::info!("Setting up proving and verification keys...");
        let (pk, vk) = client.setup(ZUNNOGAME_ELF);

        tracing::info!("Proof generator initialized");

        Ok(Self { client, pk, vk })
    }

    /// Generate proof for a game session
    pub fn generate_proof(&self, input: ProofInput) -> Result<ProofOutput> {
        tracing::info!(
            num_players = input.num_players,
            cards_per_player = input.cards_per_player,
            seed = hex::encode(input.seed),
            "Generating ZK proof"
        );

        // Prepare stdin for zkVM
        let mut stdin = SP1Stdin::new();
        stdin.write(&input.num_players);
        stdin.write(&input.cards_per_player);
        stdin.write(&input.seed);

        // Execute the program (optional - for debugging)
        tracing::debug!("Executing program...");
        let (_, report) = self
            .client
            .execute(ZUNNOGAME_ELF, &stdin)
            .run()
            .map_err(|e| anyhow!("Execution failed: {}", e))?;

        tracing::info!(
            cycles = report.total_instruction_count(),
            "Program executed successfully"
        );

        // Generate the proof
        tracing::info!("Generating compressed proof...");
        let proof = self
            .client
            .prove(&self.pk, &stdin)
            .compressed()
            .run()
            .map_err(|e| anyhow!("Proof generation failed: {}", e))?;

        tracing::info!("Proof generated successfully");

        // Convert to zkVerify-compatible format
        tracing::debug!("Converting to zkVerify format...");
        let SP1ZkvProofWithPublicValues {
            proof: shrunk_proof,
            public_values,
        } = self
            .client
            .convert_proof_to_zkv(proof, Default::default())
            .map_err(|e| anyhow!("Proof conversion failed: {}", e))?;

        let vk_hash = self.vk.hash_bytes();

        // Serialize the proof
        let serialized_proof =
            bincode::serde::encode_to_vec(&shrunk_proof, bincode::config::legacy())
                .map_err(|e| anyhow!("Proof serialization failed: {}", e))?;

        // Decode public values for returning
        let _decoded_public_values = PublicValuesStruct::abi_decode(&public_values)
            .map_err(|e| anyhow!("Failed to decode public values: {}", e))?;

        tracing::info!("Proof conversion complete");

        Ok(ProofOutput {
            proof: to_hex_with_prefix(&serialized_proof),
            image_id: to_hex_with_prefix(&vk_hash),
            pub_inputs: to_hex_with_prefix(&public_values),
        })
    }

    /// Execute only (for testing without proof generation)
    pub fn execute_only(&self, input: ProofInput) -> Result<PublicValuesStruct> {
        tracing::debug!("Executing program (no proof)");

        let mut stdin = SP1Stdin::new();
        stdin.write(&input.num_players);
        stdin.write(&input.cards_per_player);
        stdin.write(&input.seed);

        let (public_values, _) = self
            .client
            .execute(ZUNNOGAME_ELF, &stdin)
            .run()
            .map_err(|e| anyhow!("Execution failed: {}", e))?;

        let decoded = PublicValuesStruct::abi_decode(&public_values.as_slice())
            .map_err(|e| anyhow!("Failed to decode public values: {}", e))?;

        Ok(decoded)
    }
}

/// Helper function to convert bytes to hex with 0x prefix
fn to_hex_with_prefix(bytes: &[u8]) -> String {
    let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}", hex_string)
}

// For backward compatibility with the binary
pub fn generate_proof_from_inputs(
    num_players: u8,
    cards_per_player: u8,
    seed: [u8; 32],
) -> Result<ProofOutput> {
    let generator = ProofGenerator::new()?;
    generator.generate_proof(ProofInput {
        num_players,
        cards_per_player,
        seed,
    })
}
