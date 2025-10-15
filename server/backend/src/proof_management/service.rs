// backend/src/proof_management/service.rs

use reqwest::multipart;
use serde::Deserialize;

use config::IpfsProvider;

#[derive(Deserialize, Debug)]
pub struct PinataUploadResponse {
    #[serde(rename = "IpfsHash")]
    pub ipfs_hash: String,
    #[serde(rename = "PinSize")]
    pub pin_size: u64,
}

pub struct IpfsUploader {
    provider: IpfsProvider,
    client: reqwest::Client,
}

impl IpfsUploader {
    pub fn new(provider: IpfsProvider) -> Self {
        IpfsUploader {
            provider,
            client: reqwest::Client::new(),
        }
    }

    /// Upload JSON data to IPFS and return the CID
    pub async fn upload_json<T: serde::Serialize>(
        &self,
        data: &T,
        filename: &str,
    ) -> Result<String> {
        let json_str = serde_json::to_string(data)?;
        let json_bytes = json_str.as_bytes();

        self.upload_to_pinata(json_bytes, filename, api_key, api_secret).await
    }

    /// Upload to Pinata
    async fn upload_to_pinata(
        &self,
        data: &[u8],
        filename: &str,
        api_key: &str,
        api_secret: &str,
    ) -> Result<String> {
        let form = multipart::Form::new()
            .part("file", multipart::Part::bytes(data.to_vec())
                .file_name(filename.to_string()));

        let response = self.client
            .post("https://api.pinata.cloud/pinning/pinFileToIPFS")
            .basic_auth(api_key, Some(api_secret))
            .multipart(form)
            .send()
            .await
            .map_err(|e| IpfsError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(IpfsError::UploadFailed(
                format!("Pinata upload failed: {}", error_text)
            ));
        }

        let data: PinataUploadResponse = response
            .json()
            .await
            .map_err(|e| IpfsError::CidExtraction(e.to_string()))?;

        Ok(data.ipfs_hash)
    }
}
