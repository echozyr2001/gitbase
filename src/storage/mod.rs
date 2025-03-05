mod github;

use async_trait::async_trait;
use chrono::DateTime;

#[derive(Debug)]
pub struct FileMeta {
    pub sha: String,
    pub created: DateTime<chrono::Utc>,
    pub modified: DateTime<chrono::Utc>,
}

#[async_trait]
pub trait StorageBackend {
    type Error;

    async fn write(&self, path: &str, content: &str) -> Result<FileMeta, Self::Error>;
}
