// backend/src/blockchain/adapter.rs

use alloy::providers::{Identity, RootProvider};
use alloy::{
    network::Ethereum,
    primitives::Address,
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        ProviderBuilder,
    },
    transports::http::{Client, Http},
};
use anyhow::Result;
use std::env;
use url::Url;

// Type alias for the complex provider type
type HttpProvider = FillProvider<
    JoinFill<
        alloy::providers::Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
>;

pub struct BlockchainAdapter {
    pub provider: HttpProvider,
    pub contract_address: Address,
}

impl BlockchainAdapter {
    pub async fn new() -> Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_url: Url = env::var("RPC_URL")
            .map_err(|_| anyhow::anyhow!("RPC_URL not configured"))?
            .parse()?;

        let contract_address = env::var("CONTRACT_ADDRESS")
            .map_err(|_| anyhow::anyhow!("CONTRACT_ADDRESS not configured"))?;

        let provider = ProviderBuilder::new().connect_http(rpc_url);

        let contract_address = Address::parse_checksummed(&contract_address, None)?;

        Ok(Self {
            provider,
            contract_address,
        })
    }

    pub fn get_provider(&self) -> &HttpProvider {
        &self.provider
    }

    pub fn contract_address(&self) -> Address {
        self.contract_address
    }
}
