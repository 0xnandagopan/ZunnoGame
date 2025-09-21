use alloy::{
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    sol,
};
use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use std::env;

// Generate the contract bindings for the UnoGame interface
sol! {
    // The `rpc` attribute enables contract interaction via the provider.
    #[sol(rpc)]
    contract UnoGame {
        function getRandomSeed() external view returns (uint256);
    }
}

pub struct BlockchainAdapter {
    provider: Box<dyn Provider + Send + Sync>,
    contract_address: Address,
}

impl BlockchainAdapter {
    pub async fn new() -> Result<Self> {
        // Load the .env file
        dotenv().ok();

        // Initialize the provider.
        let rpc_url = env::var("RPC_URL").expect("RPC_URL not set").parse()?;
        let provider = ProviderBuilder::new().connect(rpc_url).await?;

        // Instantiate the contract instance.
        let addr = env::var("CONTRACT_ADDRESS")
            .expect("CONTRACT_ADDRESS not set")
            .parse()?;
        let contract_address: Address =
            addr.map_err(|e| anyhow!("Invalid contract address: {}", e))?;

        Ok(Self {
            provider: Box::new(provider),
            contract_address,
        })
    }

    pub async fn get_random_seed(&self) -> Result<U256> {
        let game_contract = UnoGame::new(self.contract_address, &*self.provider);

        let seed = game_contract.get_random_seed().call().await?;

        Ok(seed._0)
    }
}

// Utility function for easy access across modules
pub async fn get_random_seed() -> Result<U256> {
    let adapter = BlockchainAdapter::new().await?;
    adapter.get_random_seed().await
}
