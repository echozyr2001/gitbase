use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("GitHub storage error: {0}")]
    GitHub(#[from] GitHubStorageError),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("I/O error: {0}")]
    IoError(String),
}

#[derive(Error, Debug)]
pub enum GitHubStorageError {
    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Missing data in response: {0}")]
    MissingData(String),

    #[error("Authentication error")]
    AuthError,

    #[error("Encoding error")]
    EncodingError,

    #[error("Resource not found")]
    NotFound,

    #[error("Forbidden: insufficient permissions")]
    Forbidden,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl From<octocrab::Error> for GitHubStorageError {
    fn from(err: octocrab::Error) -> Self {
        match &err {
            octocrab::Error::GitHub { source, .. } => match source.status_code {
                http::StatusCode::NOT_FOUND => GitHubStorageError::NotFound,
                http::StatusCode::FORBIDDEN => GitHubStorageError::Forbidden,
                http::StatusCode::UNAUTHORIZED => GitHubStorageError::AuthError,
                http::StatusCode::TOO_MANY_REQUESTS => GitHubStorageError::RateLimitExceeded,
                _ => GitHubStorageError::ApiError(format!("Status code: {}", source.status_code)),
            },
            octocrab::Error::InvalidHeaderValue { .. } => {
                GitHubStorageError::ApiError("Invalid header value".into())
            }
            octocrab::Error::Http { source, .. } => {
                GitHubStorageError::ApiError(format!("HTTP error: {}", source))
            }
            octocrab::Error::Hyper { source, .. } => {
                GitHubStorageError::ApiError(format!("Hyper error: {}", source))
            }
            _ => GitHubStorageError::ApiError(format!("Unknown error: {}", err)),
        }
    }
}
