use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("GitHub storage error: {0}")]
    GitHub(#[from] GitHubStorageError),

    // 其他存储错误...
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

#[derive(Error, Debug)]
pub enum GitHubStorageError {
    #[error("GitHub API error")]
    ApiError,

    #[error("Missing data in response: {0}")]
    MissingData(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Authentication error")]
    AuthError,

    #[error("Encoding error")]
    EncodingError,
}
