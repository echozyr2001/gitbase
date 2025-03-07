use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache synchronization error")]
    SyncError,

    #[error("Document not found: {0}")]
    NotFound(String),

    #[error("Storage backend error")]
    StorageError,

    #[error("Invalid document format")]
    InvalidFormat,
}
