// backend/src/blockchain/adapter.rs

use alloy::{
    primitives::Address,
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        Provider, ProviderBuilder, RootProvider, WsConnect,
    },
};
use anyhow::{anyhow, Result};
use std::env;
use url::Url;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

// WebSocket provider type (primary)
pub type WsProvider = FillProvider<
    JoinFill<
        alloy::providers::Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
>;

// HTTP provider type (fallback)
pub type HttpProvider = FillProvider<
    JoinFill<
        alloy::providers::Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
>;

// ============================================================================
// BLOCKCHAIN ADAPTER
// ============================================================================

pub struct BlockchainAdapter {
    pub ws_provider: WsProvider,
    pub http_provider: HttpProvider,
    pub contract_address: Address,
}

impl BlockchainAdapter {
    /// Initialize adapter with both WebSocket (primary) and HTTP (fallback)
    pub async fn new() -> Result<Self> {
        dotenvy::dotenv().ok();

        // Load configuration
        let ws_rpc_url = env::var("WS_RPC_URL")
            .map_err(|_| anyhow!("WS_RPC_URL not configured (e.g., wss://...)"))?;

        let http_rpc_url: Url = env::var("HTTP_RPC_URL")
            .map_err(|_| anyhow!("HTTP_RPC_URL not configured"))?
            .parse()?;

        let contract_address =
            env::var("CONTRACT_ADDRESS").map_err(|_| anyhow!("CONTRACT_ADDRESS not configured"))?;

        // Initialize WebSocket provider (primary)
        tracing::info!(ws_url = %ws_rpc_url, "Connecting to WebSocket provider");
        let ws_connect = WsConnect::new(&ws_rpc_url);
        let ws_provider = ProviderBuilder::new()
            .connect_ws(ws_connect)
            .await
            .map_err(|e| anyhow!("Failed to connect WebSocket provider: {}", e))?;

        // Initialize HTTP provider (fallback)
        tracing::info!(http_url = %http_rpc_url, "Connecting to HTTP provider");
        let http_provider = ProviderBuilder::new().connect_http(http_rpc_url);

        let contract_address = Address::parse_checksummed(&contract_address, None)?;

        tracing::info!(
            contract = %contract_address,
            "BlockchainAdapter initialized successfully"
        );

        Ok(Self {
            ws_provider,
            http_provider,
            contract_address,
        })
    }

    /// Get WebSocket provider (for subscriptions and real-time events)
    pub fn get_ws_provider(&self) -> &WsProvider {
        &self.ws_provider
    }

    /// Get HTTP provider (for one-off queries and fallback)
    pub fn get_http_provider(&self) -> &HttpProvider {
        &self.http_provider
    }

    /// Get contract address
    pub fn contract_address(&self) -> Address {
        self.contract_address
    }
}

// ============================================================================
// CONNECTION HEALTH MONITORING
// ============================================================================

impl BlockchainAdapter {
    /// Check if WebSocket connection is healthy
    pub async fn check_ws_health(&self) -> bool {
        // Try to get block number as a health check
        self.ws_provider.get_block_number().await.is_ok()
    }

    /// Reconnect WebSocket if connection is lost
    pub async fn reconnect_ws(&mut self) -> Result<()> {
        let ws_rpc_url = env::var("WS_RPC_URL")?;

        tracing::warn!("Reconnecting WebSocket provider");

        let ws_connect = WsConnect::new(&ws_rpc_url);
        self.ws_provider = ProviderBuilder::new()
            .connect_ws(ws_connect)
            .await
            .map_err(|e| anyhow!("Failed to reconnect WebSocket: {}", e))?;

        tracing::info!("WebSocket reconnected successfully");
        Ok(())
    }
}
