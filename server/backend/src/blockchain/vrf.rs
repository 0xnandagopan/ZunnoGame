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
use std::time::Duration;
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

// ============================================================================
// VRF REQUEST OPERATIONS
// ============================================================================

impl BlockchainAdapter {
    /// Request VRF randomness from the contract
    pub async fn request_vrf(&self) -> Result<VrfRequest> {
        // Use HTTP provider for transaction sending
        let contract = UnoGame::new(self.contract_address, self.get_http_provider());

        tracing::info!("Requesting VRF randomness from contract");

        let tx_builder = contract.requestRandomWords();
        let request_id = tx_builder.call().await?;
        let block_number = self.http_provider.get_block_number().await?;

        tracing::info!(
            request_id = %request_id,
            block_number = block_number,
            "VRF request initiated"
        );

        Ok(VrfRequest {
            request_id,
            block_number,
        })
    }

    /// Create optimized event filter with indexed parameter filtering
    fn create_vrf_filter(&self, request_id: U256, from_block: u64) -> Filter {
        Filter::new()
            .address(self.contract_address)
            .event_signature(RequestFulfilled::SIGNATURE_HASH) // ✅ Type-safe
            .topic1(request_id) // ✅ Filter by indexed requestId
            .from_block(BlockNumberOrTag::Number(from_block))
    }

    /// Check if VRF event already occurred (missed event detection)
    async fn check_for_missed_event(
        &self,
        request_id: U256,
        from_block: u64,
    ) -> Result<Option<U256>> {
        tracing::debug!(
            request_id = %request_id,
            from_block = from_block,
            "Checking for missed VRF events"
        );

        let filter = self.create_vrf_filter(request_id, from_block);

        // Use HTTP provider for historical query
        let logs = self.http_provider.get_logs(&filter).await?;

        if let Some(log) = logs.first() {
            if let Ok(event) = RequestFulfilled::decode_log(&log.inner) {
                tracing::info!(
                    request_id = %request_id,
                    random_word = %event.data.randomWord,
                    "Found missed VRF event"
                );
                return Ok(Some(event.data.randomWord));
            }
        }

        Ok(None)
    }

    /// Wait for VRF event using WebSocket subscription (primary method)
    async fn wait_for_vrf_event_ws(&self, request_id: U256, from_block: u64) -> Result<U256> {
        tracing::info!(
            request_id = %request_id,
            from_block = from_block,
            "Subscribing to VRF fulfillment events via WebSocket"
        );

        let filter = self.create_vrf_filter(request_id, from_block);

        // Use WebSocket provider for real-time subscription
        let sub = self
            .ws_provider
            .subscribe_logs(&filter)
            .await
            .map_err(|e| anyhow!("Failed to subscribe to logs: {}", e))?;

        let mut stream = sub.into_stream();

        // Wait for the specific event
        while let Some(log) = stream.next().await {
            // Decode the event
            match RequestFulfilled::decode_log(&log.inner) {
                Ok(event) => {
                    let random_word = event.data.randomWord;

                    tracing::info!(
                        request_id = %request_id,
                        random_word = %random_word,
                        "VRF fulfillment received via WebSocket"
                    );

                    return Ok(random_word);
                }
                Err(e) => {
                    tracing::warn!(
                        request_id = %request_id,
                        error = %e,
                        "Failed to decode log, continuing..."
                    );
                    continue;
                }
            }
        }

        Err(anyhow!("WebSocket stream ended without receiving event"))
    }

    /// Poll for VRF result using HTTP (fallback method)
    async fn poll_random_words_http(
        &self,
        request_id: U256,
        max_attempts: u32,
        interval_secs: u64,
    ) -> Result<U256> {
        // Use HTTP provider for polling
        let contract = UnoGame::new(self.contract_address, self.get_http_provider());

        tracing::warn!(
            request_id = %request_id,
            max_attempts = max_attempts,
            interval_secs = interval_secs,
            "Falling back to HTTP polling for VRF result"
        );

        for attempt in 1..=max_attempts {
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;

            match contract.getRandomWords(request_id).call().await {
                Ok(value) if value != U256::ZERO => {
                    tracing::info!(
                        attempt = attempt,
                        request_id = %request_id,
                        value = %value,
                        "Successfully polled random word via HTTP"
                    );
                    return Ok(value);
                }
                Ok(_) => {
                    tracing::debug!(
                        attempt = attempt,
                        request_id = %request_id,
                        "VRF not ready yet, continuing..."
                    );
                    continue;
                }
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt,
                        request_id = %request_id,
                        error = %e,
                        "Polling attempt failed, retrying..."
                    );
                    continue;
                }
            }
        }

        Err(anyhow!(
            "Polling timeout after {} attempts ({} seconds)",
            max_attempts,
            max_attempts as u64 * interval_secs
        ))
    }

    /// Get VRF randomness with optimized hybrid approach
    ///
    /// Strategy:
    /// 1. Check for missed events (HTTP historical query)
    /// 2. Subscribe via WebSocket for real-time delivery (primary)
    /// 3. Fallback to HTTP polling if WebSocket fails
    pub async fn get_randomness(
        &self,
        request_id: U256,
        from_block: u64,
        timeout_secs: u64,
    ) -> Result<U256> {
        tracing::info!(
            request_id = %request_id,
            from_block = from_block,
            timeout_secs = timeout_secs,
            "Getting VRF randomness with optimized hybrid approach"
        );

        // Step 1: Check if event already occurred (prevents waiting forever)
        if let Some(random_word) = self.check_for_missed_event(request_id, from_block).await? {
            return Ok(random_word);
        }

        // Step 2: Try WebSocket subscription (primary method)
        let ws_result = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            self.wait_for_vrf_event_ws(request_id, from_block),
        )
        .await;

        match ws_result {
            Ok(Ok(random_word)) => {
                tracing::info!("VRF received via WebSocket (optimal path)");
                return Ok(random_word);
            }
            Ok(Err(e)) => {
                tracing::error!(
                    error = %e,
                    "WebSocket subscription failed, falling back to HTTP polling"
                );
            }
            Err(_) => {
                tracing::warn!("WebSocket subscription timed out, falling back to HTTP polling");
            }
        }

        // Step 3: Fallback to HTTP polling
        let poll_attempts = (timeout_secs / 3).max(5) as u32; // At least 5 attempts
        self.poll_random_words_http(request_id, poll_attempts, 3)
            .await
    }
}

// ============================================================================
// CONVENIENCE METHODS
// ============================================================================

impl BlockchainAdapter {
    /// High-level method: Request VRF and wait for result
    ///
    /// This is the main entry point for game sessions.
    pub async fn request_and_wait_for_vrf(&self, timeout_secs: u64) -> Result<U256> {
        // Step 1: Request VRF
        let vrf_request = self.request_vrf().await?;

        // Step 2: Wait for fulfillment
        let random_word = self
            .get_randomness(
                vrf_request.request_id,
                vrf_request.block_number,
                timeout_secs,
            )
            .await?;

        Ok(random_word)
    }
}
