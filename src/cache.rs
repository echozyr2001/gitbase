// use crate::github_api::{get_file_content, get_file_metadata};
use anyhow::Result;
use lru::LruCache;
use octocrab::Octocrab;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use tokio::task;
use tokio::time::{self, Duration};

/// 统一的缓存与同步管理
pub struct Cache {
    lru: Mutex<LruCache<String, (String, String)>>, // 文档 ID -> (内容, SHA)
    client: Arc<Octocrab>,
    owner: String,
    repo: String,
}

impl Cache {
    /// 创建缓存，最大存储 100 个文档，并启动自动同步任务
    pub fn new(client: Arc<Octocrab>, owner: &str, repo: &str) -> Self {
        let cache = Self {
            lru: Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())),
            client: client.clone(),
            owner: owner.to_string(),
            repo: repo.to_string(),
        };
        cache.start_auto_sync();
        cache
    }

    /// 获取文档内容（优先从缓存）
    pub async fn get(&self, collection: &str, doc_id: &str) -> Result<Option<String>> {
        let path = format!("collections/{}/{}.json", collection, doc_id);

        if let Some((content, _)) = self.lru.lock().unwrap().get(doc_id).cloned() {
            return Ok(Some(content));
        }

        // 缓存未命中，尝试同步
        self.sync_document(collection, doc_id).await?;
        Ok(self
            .lru
            .lock()
            .unwrap()
            .get(doc_id)
            .map(|(content, _)| content.clone()))
    }

    /// 存储文档到缓存（包含 SHA 值）
    pub fn insert(&self, doc_id: &str, content: &str, sha: &str) {
        let mut cache = self.lru.lock().unwrap();
        cache.put(doc_id.to_string(), (content.to_string(), sha.to_string()));
    }

    /// 删除缓存
    pub fn remove(&self, doc_id: &str) {
        let mut cache = self.lru.lock().unwrap();
        cache.pop(doc_id);
    }

    /// **手动同步**：同步指定文档
    pub async fn sync_document(&self, collection: &str, doc_id: &str) -> Result<()> {
        let path = format!("collections/{}/{}.json", collection, doc_id);

        // 1. 从缓存获取 SHA
        if let Some((_, cached_sha)) = self.lru.lock().unwrap().get(doc_id).cloned() {
            // 2. 从 GitHub 获取最新 SHA
            let github_metadata =
                get_file_metadata(&self.client, &self.owner, &self.repo, &path).await?;
            let github_sha = github_metadata.sha;

            // 3. 如果 SHA 相同，则无需同步
            if cached_sha == github_sha {
                println!("Document {} is up-to-date", doc_id);
                return Ok(());
            }
        }

        // 4. SHA 不匹配，获取最新内容
        if let Some(content) =
            get_file_content(&self.client, &self.owner, &self.repo, &path).await?
        {
            self.insert(doc_id, &content, &github_metadata.sha);
            println!("Document {} synchronized", doc_id);
        }

        Ok(())
    }

    /// **全局同步**：同步整个 GitBase
    pub async fn sync_all_documents(&self) -> Result<()> {
        let collections = vec!["notes", "tasks"]; // 可动态获取
        for collection in collections {
            let doc_ids = vec!["doc1", "doc2"]; // 假设获取所有文档 ID
            for doc_id in doc_ids {
                self.sync_document(collection, doc_id).await?;
            }
        }
        Ok(())
    }

    /// 启动自动同步任务，每 10 分钟同步一次缓存与 GitHub
    pub fn start_auto_sync(&self) {
        let client = self.client.clone();
        let owner = self.owner.clone();
        let repo = self.repo.clone();
        let cache = Arc::new(self.clone());

        task::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(600)); // 每 10 分钟同步一次
            loop {
                interval.tick().await;
                println!("Starting automatic cache sync...");
                if let Err(e) = cache.sync_all_documents().await {
                    eprintln!("Auto-sync failed: {:?}", e);
                }
            }
        });
    }
}
