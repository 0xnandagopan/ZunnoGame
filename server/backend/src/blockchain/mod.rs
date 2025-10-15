// backend/src/blockchain/mod.rs

pub mod adapter;
pub mod types;
pub mod vrf;

pub use adapter::BlockchainAdapter;
pub use types::BlockchainSeed;
pub use vrf::VrfRequest;

// Re-export for convenience
pub use alloy::primitives::U256;
