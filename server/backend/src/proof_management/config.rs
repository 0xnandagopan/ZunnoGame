// backend/src/proof_management/config.rs

use pinata_sdk::PinataApi;
use std::env;

#[derive(Clone, Debug)]
pub struct IpfsProvider {
    pinata_api: PinataApi,
}

impl IpfsProvider {
    pub fn from_env() -> Result<Self> {
        if let (Ok(key), Ok(secret)) = (env::var("PINATA_API_KEY"), env::var("PINATA_API_SECRET")) {
            let api = PinataApi::new(key, secret).unwrap();
            return Ok(IpfsProvider { pinata_api });
        }
    }
}
