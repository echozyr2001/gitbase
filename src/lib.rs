#![allow(dead_code)]

mod coder;

use anyhow::Result;
use lru::LruCache;
use octocrab::models::repos::Content;
use octocrab::Octocrab;
use serde_json::Value;
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct GitBase {
    client: Arc<Octocrab>,
    cache: Arc<Mutex<LruCache<String, (String, String)>>>,
    owner: String,
    repo: String,
}

impl GitBase {
    /// 创建 GitBase 实例，初始化缓存和 GitHub 客户端
    pub fn new(token: &str, owner: &str, repo: &str) -> Arc<Self> {
        let client = Arc::new(
            Octocrab::builder()
                .personal_token(token.to_string())
                .build()
                .unwrap(),
        );

        Arc::new(Self {
            client,
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    /// 从 GitHub 仓库中获取文件内容
    pub async fn fetch_file(&self, path: &str, branch: &str) -> Result<Content> {
        let mut contents = self
            .client
            .repos(&self.owner, &self.repo)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await
            .unwrap();
        let contents = contents.take_items();
        let content = &contents[0];

        Ok(content.clone())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub id: String,
    pub content: Value,
    pub meta: Metadata,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    pub created_at: String,
    pub updated_sha: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv_codegen::dotenv;

    fn init_gitbase() -> Arc<GitBase> {
        let token = dotenv!("GB_GITHUB_TOKEN");
        let owner = dotenv!("GB_GITHUB_OWNER");
        let repo = dotenv!("GB_GITHUB_REPO");

        GitBase::new(token, owner, repo)
    }

    #[tokio::test]
    async fn test_fetch_github_file() {
        let gitbase = init_gitbase();

        let content = gitbase
            .fetch_file("README.md", "main")
            .await
            .unwrap()
            .decoded_content()
            .unwrap();
        println!("content: {:?}", content);
    }
}
