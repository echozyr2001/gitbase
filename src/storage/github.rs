use super::{FileMeta, StorageBackend};
use async_trait::async_trait;
use octocrab::Octocrab;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHubStorageError {
    #[error("GitHub API error: {0}")]
    ApiError(#[from] octocrab::Error),

    #[error("Missing data in response: {0}")]
    MissingData(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Authentication error: {0}")]
    AuthError(String),
}

impl fmt::Display for GitHubStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GitHubStorage(owner={}, repo={}, branch={})",
            self.owner, self.repo, self.branch
        )
    }
}

pub struct GitHubStorage {
    client: Octocrab,
    owner: String,
    repo: String,
    branch: String,
}

impl GitHubStorage {
    pub fn new(
        token: &str,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
    ) -> Result<Self, GitHubStorageError> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| GitHubStorageError::AuthError(e.to_string()))?;

        Ok(GitHubStorage {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch: branch.unwrap_or("main").to_string(),
        })
    }
}

#[async_trait]
impl StorageBackend for GitHubStorage {
    type Error = GitHubStorageError;

    async fn write(&self, path: &str, content: &str) -> Result<FileMeta, Self::Error> {
        if path.is_empty() {
            return Err(GitHubStorageError::InvalidPath(
                "Path cannot be empty".into(),
            ));
        }

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
                // File exists, update it
                let item = content_info.items.first().ok_or_else(|| {
                    GitHubStorageError::MissingData("No content items found".into())
                })?;

                let sha = item.sha.clone();

                // Store the original creation date by getting the first commit for this file
                let commits = self
                    .client
                    .repos(&self.owner, &self.repo)
                    .list_commits()
                    .path(path)
                    .per_page(1) // We only need the first commit
                    .send()
                    .await
                    .map_err(GitHubStorageError::ApiError)?;

                // Get the first (earliest) commit for this file
                let first_commit = commits.items.first().ok_or_else(|| {
                    GitHubStorageError::MissingData("No commit history found for file".into())
                })?;

                let created = first_commit
                    .commit
                    .author
                    .as_ref()
                    .ok_or_else(|| {
                        GitHubStorageError::MissingData(
                            "Missing author data for original file".into(),
                        )
                    })?
                    .date
                    .ok_or_else(|| {
                        GitHubStorageError::MissingData(
                            "Missing author date for original file".into(),
                        )
                    })?;

                let commit = self
                    .client
                    .repos(&self.owner, &self.repo)
                    .update_file(path, format!("Update {}", path), content, &sha)
                    .branch(&self.branch)
                    .send()
                    .await?;

                let modified = commit
                    .commit
                    .committer
                    .ok_or_else(|| {
                        GitHubStorageError::MissingData("Missing committer data".into())
                    })?
                    .date
                    .ok_or_else(|| {
                        GitHubStorageError::MissingData("Missing committer date".into())
                    })?;

                Ok(FileMeta {
                    sha: commit.content.sha,
                    created,  // Use the original creation date
                    modified, // Use the new modification date
                })
            }
            Err(_) => {
                // File doesn't exist, create it
                let commit = self
                    .client
                    .repos(&self.owner, &self.repo)
                    .create_file(path, format!("Create {}", path), content)
                    .branch(&self.branch)
                    .send()
                    .await?;

                // For new files, both created and modified are the same
                let created = commit
                    .commit
                    .author
                    .ok_or_else(|| GitHubStorageError::MissingData("Missing author data".into()))?
                    .date
                    .ok_or_else(|| GitHubStorageError::MissingData("Missing author date".into()))?;

                let modified = commit
                    .commit
                    .committer
                    .ok_or_else(|| {
                        GitHubStorageError::MissingData("Missing committer data".into())
                    })?
                    .date
                    .ok_or_else(|| {
                        GitHubStorageError::MissingData("Missing committer date".into())
                    })?;

                Ok(FileMeta {
                    sha: commit.content.sha,
                    created,
                    modified,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv_codegen::dotenv;

    fn init_gitbase() -> Result<GitHubStorage, GitHubStorageError> {
        let token = dotenv!("GB_GITHUB_TOKEN");
        let owner = dotenv!("GB_GITHUB_OWNER");
        let repo = dotenv!("GB_GITHUB_REPO");

        GitHubStorage::new(token, owner, repo, Some("main"))
    }

    #[tokio::test]
    async fn test_write_file() {
        let gitbase = init_gitbase().expect("Failed to initialize GitHubStorage");

        let content = "Hello, world!";
        let result = gitbase.write("test.txt", content).await;
        match result {
            Ok(meta) => println!("meta: {:?}", meta),
            Err(e) => panic!("Error writing file: {}", e),
        }
    }
}
