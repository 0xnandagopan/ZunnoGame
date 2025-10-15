// backend/src/proof_management/config.rs

use std::env;

#[derive(Clone, Debug)]
pub enum IpfsProvider {
    Pinata { api_key: String, api_secret: String },
}

impl IpfsProvider {
    pub fn from_env() -> Result<Self> {
        if let (Ok(key), Ok(secret)) = (env::var("PINATA_API_KEY"), env::var("PINATA_API_SECRET")) {
            return Ok(IpfsProvider::Pinata {
                api_key: key,
                api_secret: secret,
            });
        }
    }
}
