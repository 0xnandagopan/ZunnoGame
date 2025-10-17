// backend/src/proof_management/config.rs

use super::errors::{IpfsError, IpfsResult};
use pinata_sdk::PinataApi;
use std::env;

pub struct IpfsProvider {
    pub pinata_api: PinataApi,
}

impl IpfsProvider {
    pub fn from_env() -> IpfsResult<Self> {
        if let (Ok(key), Ok(secret)) = (env::var("PINATA_API_KEY"), env::var("PINATA_API_SECRET")) {
            let api = PinataApi::new(key, secret).unwrap();
            return Ok(IpfsProvider { pinata_api: api });
        } else {
            return Err(IpfsError::ConfigError(format!(
                "Missing environment variables"
            )));
        }
    }
}
