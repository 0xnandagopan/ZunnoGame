// backend/src/proof_management/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpfsError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("CID extraction failed: {0}")]
    CidExtraction(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type IpfsResult<T> = Result<T, IpfsError>;
