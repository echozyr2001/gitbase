use super::{FileMeta, StorageBackend};
use async_trait::async_trait;
use octocrab::Octocrab;

pub struct GitHubStorage {
    client: Octocrab,
    owner: String,
    repo: String,
    branch: String,
}

impl GitHubStorage {
    pub fn new(token: &str, owner: &str, repo: &str, branch: Option<&str>) -> Self {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .unwrap();

        GitHubStorage {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch: branch.unwrap_or("main").to_string(),
        }
    }
}

#[async_trait]
impl StorageBackend for GitHubStorage {
    type Error = octocrab::Error; // Add associated error type

    async fn write(&self, path: &str, content: &str) -> Result<FileMeta, Self::Error> {
        let get_result = self
            .client
            .repos(&self.owner, &self.repo)
            .get_content()
            .path(path)
            .r#ref(&self.branch)
            .send()
            .await;

        match get_result {
            Ok(content_info) => {
                // 文件存在，执行更新操作
                let sha = content_info.items[0].sha.clone();
                let commit = self
                    .client
                    .repos(&self.owner, &self.repo)
                    .update_file(path, format!("Update {}", path), content, &sha)
                    .branch(&self.branch)
                    .send()
                    .await?;

                Ok(FileMeta {
                    sha: commit.content.sha,
                    created: commit.commit.author.unwrap().date.unwrap(),
                    modified: commit.commit.committer.unwrap().date.unwrap(),
                })
            }
            Err(_) => {
                // 文件不存在，执行创建操作
                let commit = self
                    .client
                    .repos(&self.owner, &self.repo)
                    .create_file(path, format!("Create {}", path), content)
                    .branch(&self.branch)
                    .send()
                    .await?;

                Ok(FileMeta {
                    sha: commit.content.sha,
                    created: commit.commit.author.unwrap().date.unwrap(),
                    modified: commit.commit.committer.unwrap().date.unwrap(),
                })
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use dotenv_codegen::dotenv;

//     fn init_gitbase() -> GitHubStorage {
//         let token = dotenv!("GB_GITHUB_TOKEN");
//         let owner = dotenv!("GB_GITHUB_OWNER");
//         let repo = dotenv!("GB_GITHUB_REPO");

//         GitHubStorage::new(token, owner, repo, Some("main"))
//     }

//     #[tokio::test]
//     async fn test_write_file() {
//         let gitbase = init_gitbase();

//         let content = "Hello, world!";
//         let meta = gitbase.write("test.txt", content).await.unwrap();
//         println!("meta: {:?}", meta);
//     }
// }
