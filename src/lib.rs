#![allow(dead_code)]

mod coder;
mod error;

use anyhow::{Ok, Result};
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

    pub async fn create_collection(&self, name: &str, branch: &str) -> Result<()> {
        let collection_id = coder::generate_collection_id(name).unwrap();
        let dir_path = format!("collections/{}", name);

        // 1. 在集合目录下创建 `.gitkeep` 文件，让 Git 识别目录
        let gitkeep_path = format!("{}/.gitkeep", dir_path);
        self.client
            .repos(&self.owner, &self.repo)
            .create_file(gitkeep_path, "Initialize collection directory", "")
            .branch(branch)
            .send()
            .await?;

        // 2. 在集合目录下创建 `collection.json`，存储唯一 ID
        let metadata_path = format!("{}/collection.json", dir_path);
        let metadata_content = serde_json::json!({
            "name": name,
            "collection_id": collection_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
        })
        .to_string();

        self.client
            .repos(&self.owner, &self.repo)
            .create_file(
                metadata_path,
                "Store collection metadata",
                &metadata_content,
            )
            .branch(branch)
            .send()
            .await?;

        Ok(())
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use dotenv_codegen::dotenv;

//     fn init_gitbase() -> Arc<GitBase> {
//         let token = dotenv!("GB_GITHUB_TOKEN");
//         let owner = dotenv!("GB_GITHUB_OWNER");
//         let repo = dotenv!("GB_GITHUB_REPO");

//         GitBase::new(token, owner, repo)
//     }

//     #[tokio::test]
//     async fn test_fetch_github_file() {
//         let gitbase = init_gitbase();

//         let content = gitbase
//             .fetch_file("README.md", "main")
//             .await
//             .unwrap()
//             .decoded_content()
//             .unwrap();
//         println!("content: {:?}", content);
//     }

//     #[tokio::test]
//     async fn test_create_collection() {
//         let gitbase = init_gitbase();

//         gitbase.create_collection("notes", "main").await.unwrap();
//     }
// }
