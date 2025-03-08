mod github;

use async_trait::async_trait;
use chrono::DateTime;

use crate::error::StorageResult;

#[derive(Debug)]
pub struct FileMeta {
    pub sha: String,
    pub created: DateTime<chrono::Utc>,
    pub modified: DateTime<chrono::Utc>,
}

#[async_trait]
pub trait StorageBackend {
    async fn write(&self, path: &str, content: &str) -> StorageResult<FileMeta>;

    async fn read(&self, path: &str) -> StorageResult<String>;
}
