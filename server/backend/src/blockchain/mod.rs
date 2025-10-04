use alloy::{
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    sol,
};
use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use rand::Rng;
use std::env;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};
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

        let rpc_url_var = env::var("RPC_URL").ok();
        let contract_addr_var = env::var("CONTRACT_ADDRESS").ok();

        if rpc_url_var.is_none() || contract_addr_var.is_none() {
            let fallback_seed: u64 = rand::thread_rng().gen();
            warn!(
                seed = fallback_seed,
                "RPC_URL or CONTRACT_ADDRESS missing; using locally generated seed"
            );
            return Ok(U256::from(fallback_seed));
        }

        let rpc_url_raw = rpc_url_var.unwrap();
        let contract_addr_raw = contract_addr_var.unwrap();

        let rpc_url: Url = match rpc_url_raw.parse() {
            Ok(url) => url,
            Err(e) => {
                let fallback_seed: u64 = rand::thread_rng().gen();
                warn!(error = %e, url = %rpc_url_raw, seed = fallback_seed, "Invalid RPC_URL; using locally generated seed");
                return Ok(U256::from(fallback_seed));
            }
        };

        let provider = ProviderBuilder::new().connect_http(rpc_url);

        let contract_address = match Address::parse_checksummed(&contract_addr_raw, None) {
            Ok(addr) => addr,
            Err(e) => {
                let fallback_seed: u64 = rand::thread_rng().gen();
                warn!(error = %e, address = %contract_addr_raw, seed = fallback_seed, "Invalid CONTRACT_ADDRESS; using locally generated seed");
                return Ok(U256::from(fallback_seed));
            }
        };

        let game_contract = UnoGame::new(contract_address, provider);

        let _req_id: U256 = match game_contract.requestRandomWords().call().await {
            Ok(req_id) => {
                info!("Requested random words from contract");
                req_id
            }
            Err(e) => {
                let fallback_seed: u64 = rand::thread_rng().gen();
                warn!(error = %e, seed = fallback_seed, "Failed to request random words; using locally generated seed");
                return Ok(U256::from(fallback_seed));
            }
        };

        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 3;
        while attempts < MAX_ATTEMPTS {
            sleep(Duration::from_secs(3)).await;
            attempts += 1;
            match game_contract.getRandomWords().call().await {
                Ok(seed) => {
                    info!(attempt = attempts, "Received random seed from blockchain");
                    return Ok(seed);
                }
                Err(e) => {
                    warn!(attempt = attempts, error = %e, "Attempt to fetch random seed failed");
                    if attempts >= MAX_ATTEMPTS {
                        break;
                    }
                }
            }
        }

        let fallback_seed: u64 = rand::thread_rng().gen();
        warn!(
            seed = fallback_seed,
            "Exceeded max attempts fetching blockchain seed; using locally generated seed"
        );
        Ok(U256::from(fallback_seed))
    }
}

// Utility function for easy access across modules
pub async fn get_random_seed() -> Result<U256> {
    let adapter = BlockchainAdapter::new().await?;
    adapter.get_random_seed().await
}
