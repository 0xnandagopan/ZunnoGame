// backend/src/proof_management/service.rs

use pinata_sdk::{
    // ApiError,
    PinByJson,
    PinataApi,
};
use serde::Deserialize;

use super::config::IpfsProvider;
use super::errors::{IpfsError, IpfsResult};

#[derive(Deserialize, Debug)]
pub struct PinataUploadResponse {
    #[serde(rename = "IpfsHash")]
    pub ipfs_hash: String,
    #[serde(rename = "PinSize")]
    pub pin_size: u64,
}

pub struct IpfsUploader {
    pub api: PinataApi,
}

impl IpfsUploader {
    pub fn new(provider: IpfsProvider) -> Self {
        let api = provider.pinata_api;
        IpfsUploader { api }
    }

    /// Upload JSON data to IPFS and return the CID
    pub async fn upload_json<T: serde::Serialize>(&self, data: &T) -> IpfsResult<String> {
        let json_data = serde_json::to_string_pretty(data)?;

        match self.api.pin_json(PinByJson::new(json_data)).await {
            Ok(pinned_object) => {
                return Ok(pinned_object.ipfs_hash);
            }
            Err(e) => {
                return Err(IpfsError::UploadFailed(format!(
                    "Pinata upload failed: {}",
                    e
                )));
            }
        }
    }
}
