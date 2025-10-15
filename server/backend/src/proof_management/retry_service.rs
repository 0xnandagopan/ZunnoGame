// backend/src/proof_management/retry_service.rs

use std::time::Duration;

use config::IpfsProvider;
use service::IpfsUploader;

#[derive(Clone)]
pub struct IpfsUploadConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for IpfsUploadConfig {
    fn default() -> Self {
        IpfsUploadConfig {
            max_retries: 3,
            retry_delay: Duration::from_secs(2),
        }
    }
}

pub struct IpfsService {
    uploader: IpfsUploader,
    config: IpfsUploadConfig,
}

impl IpfsService {
    pub fn new(provider: IpfsProvider, config: IpfsUploadConfig) -> Self {
        IpfsService {
            uploader: IpfsUploader::new(provider),
            config,
        }
    }

    /// Upload with automatic retry on failure
    pub async fn upload_with_retry<T: serde::Serialize>(
        &self,
        data: &T,
        filename: &str,
    ) -> IpfsResult<String> {
        let mut attempts = 0;

        loop {
            match self.uploader.upload_json(data, filename).await {
                Ok(cid) => {
                    println!("✓ Upload successful. CID: {}", cid);
                    return Ok(cid);
                }
                Err(e) if attempts < self.config.max_retries => {
                    attempts += 1;
                    eprintln!(
                        "⚠ Upload attempt {} failed: {}. Retrying in {:?}...",
                        attempts, e, self.config.retry_delay
                    );
                    tokio::time::sleep(self.config.retry_delay).await;
                }
                Err(e) => {
                    eprintln!("✗ Upload failed after {} attempts: {}", attempts + 1, e);
                    return Err(e);
                }
            }
        }
    }
}
