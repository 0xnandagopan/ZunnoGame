use alloy::{
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    sol,
};
use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use std::env;
use tokio::time::{sleep, Duration};
use url::Url;

// Generate the contract bindings for the UnoGame interface
sol! {
    // The `rpc` attribute enables contract interaction via the provider.
    #[sol(rpc)]
    contract UnoGame {
        function requestRandomWords() public returns(uint256);
        function getRandomWords() public view returns (uint256);
    }
}

pub struct BlockchainAdapter;

impl BlockchainAdapter {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub async fn get_random_seed(&self) -> Result<U256> {
        dotenv().ok();

        // Initialize the provider.
        let rpc_url_string = env::var("RPC_URL").expect("RPC_URL not set");
        let rpc_url: Url = rpc_url_string.parse()?;
        let provider = ProviderBuilder::new().connect_http(rpc_url);

        // Instantiate the contract instance.
        let addr_string = env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS not set");
        let contract_address = Address::parse_checksummed(&addr_string, None)
            .map_err(|e| anyhow!("Invalid contract address '{}': {}", addr_string, e))?;

        let game_contract = UnoGame::new(contract_address, provider);

        let req_id: U256 = game_contract.requestRandomWords().call().await?;

        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 3;
        while attempts < MAX_ATTEMPTS {
            sleep(Duration::from_secs(3)).await;
            attempts += 1;
            match game_contract.getRandomWords().call().await {
                Ok(seed) => {
                    println!("Received random seed");
                    return Ok(seed);
                }
                Err(e) => {
                    println!("Attempt {} failed: {}", attempts, e);
                    if attempts >= MAX_ATTEMPTS {
                        break;
                    }
                }
            }
        }

        Err(anyhow!(
            "Failed to get random seed after {} attempts",
            MAX_ATTEMPTS
        ))
    }
}

// Utility function for easy access across modules
pub async fn get_random_seed() -> Result<U256> {
    let adapter = BlockchainAdapter::new().await?;
    adapter.get_random_seed().await
}
