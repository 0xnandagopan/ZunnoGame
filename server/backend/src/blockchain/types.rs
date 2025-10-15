// backend/src/blockchain/types.rs

use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainSeed {
    pub value: U256,
    pub request_id: U256,
}

impl Default for BlockchainSeed {
    fn default() -> Self {
        Self {
            value: U256::ZERO,
            request_id: U256::ZERO,
        }
    }
}

pub fn u256_to_bytes32(value: U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&value.to_be_bytes::<32>());
    bytes
}
