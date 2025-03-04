use thiserror::Error;

#[derive(Error, Debug)]
pub enum Bech32Error {
    #[error("Invalid HRP format")]
    InvalidHRP,

    #[error("Failed to encode Bech32: {0}")]
    EncodingError(String),

    #[error("Failed to decode Bech32: {0}")]
    DecodingError(String),

    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON Serialization Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Timeout while waiting for async task")]
    AsyncTaskTimeout,

    #[error("Ali API Error: {0}")]
    AliApiError(String),
}
