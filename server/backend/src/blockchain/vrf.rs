// backend/src/blockchain/vrf.rs

use super::adapter::BlockchainAdapter;
use alloy::{
    primitives::U256,
    providers::Provider,
    rpc::types::{BlockNumberOrTag, Filter},
    sol,
};
use alloy_sol_types::SolEvent;
use anyhow::{anyhow, Result};
use std::{time::Duration, u64};
use tokio_stream::StreamExt;

// ============================================================================
// CONTRACT ABI DEFINITIONS
// ============================================================================

sol! {
    #[sol(rpc)]
    contract UnoGame {
        function requestRandomWords() public returns(uint256);
        function getRandomWords(uint256 requestId) public view returns (uint256);
    }

    /// Event emitted when VRF request is fulfilled
    #[derive(Debug)]
    event RequestFulfilled(
        uint256 indexed requestId,
        uint256 randomWord
    );
}

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

#[derive(Debug, Clone)]
pub struct VrfRequest {
    pub request_id: U256,
    pub block_number: u64,
}

impl BlockchainAdapter {
    /// Request VRF randomness from the contract
    pub async fn request_vrf(&self) -> Result<VrfRequest> {
        let contract = UnoGame::new(self.contract_address, &self.provider);

        tracing::info!("Requesting VRF randomness from contract");

        let tx_builder = contract.requestRandomWords();
        let request_id = tx_builder.call().await?;
        let block_number = self.provider.get_block_number().await.unwrap();

        tracing::info!(
            request_id = %request_id,
            block_number = block_number,
            "VRF request successful"
        );

        Ok(VrfRequest {
            request_id,
            block_number,
        })
    }

    /// Create event filter for VRF fulfillments
    pub fn create_vrf_filter(&self, block_number: u64) -> Filter {
        Filter::new()
            .address(self.contract_address)
            .event("RequestFulfilled(uint256, uint256)")
            .from_block(BlockNumberOrTag::Number(block_number))
    }

    pub async fn wait_for_vrf_event(&self, request_id: U256, block_number: u64) -> Result<U256> {
        tracing::info!(
            request_id = %request_id,
            from_block = block_number,
            "Waiting for VRF fulfillment event..."
        );

        let filter = self.create_vrf_filter(block_number);
        let sub = self
            .provider
            .subscribe_logs(&filter)
            .await
            .map_err(|e| anyhow!("Failed to subscribe to logs: {}", e))?;

        let mut stream = sub.into_stream();

        while let Some(log) = stream.next().await {
            if let Ok(event) = RequestFulfilled::decode_log(log) {
                let data = event.data;
                if data.requestId == request_id {
                    tracing::info!(
                        request_id = %request_id,
                        random_word = %data.randomWord,
                        "VRF fulfillment event matched!"
                    );

                    return Ok(data.randomWord);
                }
            }
        }

        Err(anyhow!("Event stream ended without receiving fulfillment"))
    }

    /// Poll for VRF result
    pub async fn poll_random_words(&self, request_id: U256) -> Result<U256> {
        let contract = UnoGame::new(self.contract_address, &self.provider);

        const MAX_ATTEMPTS: u32 = 20;
        const POLL_INTERVAL_SECS: u64 = 3;

        tracing::info!(
            request_id = %request_id,
            max_attempts = MAX_ATTEMPTS,
            interval_secs = POLL_INTERVAL_SECS,
            "Starting polling for random words..."
        );

        for attempt in 1..=MAX_ATTEMPTS {
            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

            match contract.getRandomWords(request_id).call().await {
                Ok(value) if value != U256::ZERO => {
                    tracing::info!(
                        attempt = attempt,
                        request_id = %request_id,
                        value = %value,
                        "Successfully polled random word"
                    );
                    return Ok(value);
                }
                _ => continue,
            }
        }

        Err(anyhow!(
            "Polling timeout after {} attempts ({} seconds)",
            MAX_ATTEMPTS,
            MAX_ATTEMPTS as u64 * POLL_INTERVAL_SECS
        ))
    }

    /// Hybrid approach: Try event listening with polling fallback (simplified)
    pub async fn get_randomness(
        &self,
        request_id: U256,
        block_number: u64,
        timeout_secs: u64,
    ) -> Result<U256> {
        tracing::info!(
            request_id = %request_id,
            block_number = block_number,
            timeout_secs = timeout_secs,
            "Getting randomness with hybrid approach"
        );

        // Apply timeout to the entire operation
        tokio::time::timeout(Duration::from_secs(timeout_secs), async {
            tokio::select! {
                // Try event listening
                result = self.wait_for_vrf_event(request_id, block_number) => {
                    match result {
                        Ok(value) => {
                            tracing::info!("Got random value from event listener");
                            Ok(value)
                        }
                        Err(e) => {
                            tracing::error!("Event listener failed: {}", e);
                            Err(e)
                        }
                    }
                }
                // Try polling
                result = self.poll_random_words(request_id) => {
                    match result {
                        Ok(value) => {
                            tracing::warn!("Got random value from polling");
                            Ok(value)
                        }
                        Err(e) => {
                            tracing::error!("Polling failed: {}", e);
                            Err(e)
                        }
                    }
                }
            }
        })
        .await
        .map_err(|_| anyhow!("Timeout after {} seconds", timeout_secs))?
    }
}
