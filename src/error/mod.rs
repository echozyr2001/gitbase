mod cache;
mod coder;
mod storage;

use thiserror::Error;

pub use cache::CacheError;
pub use coder::CoderError;
pub use storage::{GitHubStorageError, StorageError};

#[derive(Error, Debug)]
pub enum GBError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Coder error: {0}")]
    Coder(#[from] CoderError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Other(String),
}

pub type CacheResult<T> = Result<T, CacheError>;
pub type AppResult<T> = error_stack::Result<T, GBError>;
pub type CoderResult<T> = error_stack::Result<T, CoderError>;
pub type StorageResult<T> = error_stack::Result<T, StorageError>;
pub type GitHubStorageResult<T> = error_stack::Result<T, GitHubStorageError>;

// pub trait ErrorExt<T, E> {
//     fn with_context<C, F>(self, context_provider: F) -> Result<T, C>
//     where
//         C: std::error::Error,
//         E: std::error::Error + Into<C>,
//         F: FnOnce() -> C;
// }

// impl<T, E> ErrorExt<T, E> for std::result::Result<T, E>
// where
//     E: std::error::Error,
// {
//     fn with_context<C, F>(self, context_provider: F) -> Result<T, C>
//     where
//         C: std::error::Error,
//         E: std::error::Error + Into<C>,
//         F: FnOnce() -> C,
//     {
//         match self {
//             Ok(value) => Ok(value),
//             Err(error) => {
//                 let context = context_provider();
//                 let report = Report::new(error.into());
//                 Err(report)
//             }
//         }
//     }
// }
