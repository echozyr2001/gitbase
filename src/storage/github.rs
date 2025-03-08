use crate::error::{GitHubStorageError, GitHubStorageResult, StorageError, StorageResult};

use super::{FileMeta, StorageBackend};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use error_stack::Report;
use octocrab::Octocrab;
use std::fmt;

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
    ) -> GitHubStorageResult<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| {
                Report::new(GitHubStorageError::AuthError).attach_printable(e.to_string())
            })?;

        Ok(GitHubStorage {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch: branch.unwrap_or("main").to_string(),
        })
    }

    async fn create_file(&self, path: &str, content: &str) -> StorageResult<FileMeta> {
        let commit = self
            .client
            .repos(&self.owner, &self.repo)
            .create_file(path, format!("Create {}", path), content)
            .branch(&self.branch)
            .send()
            .await
            .map_err(|e| {
                Report::new(StorageError::GitHub(GitHubStorageError::ApiError))
                    .attach_printable(format!("Failed to create file: {}", e))
            })?;

        // For new files, both created and modified are the same
        let created = commit
            .commit
            .author
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing author data".into(),
                )))
            })?
            .date
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing author date".into(),
                )))
            })?;

        let modified = commit
            .commit
            .committer
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing committer data".into(),
                )))
            })?
            .date
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing committer date".into(),
                )))
            })?;

        Ok(FileMeta {
            sha: commit.content.sha,
            created,
            modified,
        })
    }
}

#[async_trait]
impl StorageBackend for GitHubStorage {
    async fn write(&self, path: &str, content: &str) -> StorageResult<FileMeta> {
        if path.is_empty() {
            return Err(Report::new(StorageError::InvalidPath(
                "Path cannot be empty".into(),
            )));
        }

        // Try to get existing content, handle NotFound errors separately
        let get_result = match self
            .client
            .repos(&self.owner, &self.repo)
            .get_content()
            .path(path)
            .r#ref(&self.branch)
            .send()
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => {
                // If the file doesn't exist, create it
                if e.to_string().contains("404") {
                    // File doesn't exist, create it
                    return self.create_file(path, content).await;
                }
                // Otherwise, propagate the error
                Err(
                    Report::new(StorageError::GitHub(GitHubStorageError::ApiError))
                        .attach_printable(format!("Failed to get content: {}", e)),
                )
            }
        }?;

        // File exists, check if content has changed
        let item = get_result.items.first().ok_or_else(|| {
            Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                "No content items found".into(),
            )))
        })?;

        let sha = item.sha.clone();

        // Check if content has changed by comparing with current content
        if let Some(encoded_content) = &item.content {
            // GitHub API returns base64 encoded content with possible newlines
            let cleaned_encoded = encoded_content.replace("\n", "");

            // Decode the content
            let current_content =
                general_purpose::STANDARD
                    .decode(&cleaned_encoded)
                    .map_err(|e| {
                        Report::new(StorageError::GitHub(GitHubStorageError::EncodingError))
                            .attach_printable(format!("Failed to decode content: {}", e))
                    })?;

            let current_content_str = String::from_utf8(current_content).map_err(|e| {
                Report::new(StorageError::GitHub(GitHubStorageError::EncodingError))
                    .attach_printable(format!("Failed to convert bytes to UTF-8: {}", e))
            })?;

            // If content hasn't changed, return early with existing metadata
            if current_content_str == content {
                // Get the creation date
                let commits = self
                    .client
                    .repos(&self.owner, &self.repo)
                    .list_commits()
                    .path(path)
                    .per_page(1)
                    .send()
                    .await
                    .map_err(|e| {
                        Report::new(StorageError::GitHub(GitHubStorageError::ApiError))
                            .attach_printable(format!("Failed to list commits: {}", e))
                    })?;

                let first_commit = commits.items.first().ok_or_else(|| {
                    Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                        "No commit history found for file".into(),
                    )))
                })?;

                let created = first_commit
                    .commit
                    .author
                    .as_ref()
                    .ok_or_else(|| {
                        Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                            "Missing author data for original file".into(),
                        )))
                    })?
                    .date
                    .ok_or_else(|| {
                        Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                            "Missing author date for original file".into(),
                        )))
                    })?;

                // For modified date, use the latest commit date
                let modified = first_commit
                    .commit
                    .committer
                    .as_ref()
                    .ok_or_else(|| {
                        Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                            "Missing committer data for file".into(),
                        )))
                    })?
                    .date
                    .ok_or_else(|| {
                        Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                            "Missing committer date for file".into(),
                        )))
                    })?;

                return Ok(FileMeta {
                    sha: item.sha.clone(),
                    created,
                    modified,
                });
            }
        }

        // Content has changed, proceed with update
        // First get the original creation date
        let commits = self
            .client
            .repos(&self.owner, &self.repo)
            .list_commits()
            .path(path)
            .per_page(1)
            .send()
            .await
            .map_err(|e| {
                Report::new(StorageError::GitHub(GitHubStorageError::ApiError))
                    .attach_printable(format!("Failed to list commits: {}", e))
            })?;

        let first_commit = commits.items.first().ok_or_else(|| {
            Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                "No commit history found for file".into(),
            )))
        })?;

        let created = first_commit
            .commit
            .author
            .as_ref()
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing author data for original file".into(),
                )))
            })?
            .date
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing author date for original file".into(),
                )))
            })?;

        // Update the file
        let commit = self
            .client
            .repos(&self.owner, &self.repo)
            .update_file(path, format!("Update {}", path), content, &sha)
            .branch(&self.branch)
            .send()
            .await
            .map_err(|e| {
                Report::new(StorageError::GitHub(GitHubStorageError::ApiError))
                    .attach_printable(format!("Failed to update file: {}", e))
            })?;

        let modified = commit
            .commit
            .committer
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing committer data".into(),
                )))
            })?
            .date
            .ok_or_else(|| {
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "Missing committer date".into(),
                )))
            })?;

        Ok(FileMeta {
            sha: commit.content.sha,
            created,
            modified,
        })
    }

    async fn read(&self, path: &str) -> StorageResult<String> {
        let get_result = self
            .client
            .repos(&self.owner, &self.repo)
            .get_content()
            .path(path)
            .r#ref(&self.branch)
            .send()
            .await
            .map_err(|e| {
                Report::new(StorageError::GitHub(GitHubStorageError::ApiError))
                    .attach_printable(format!("Failed to get content: {}", e))
            })?;

        let item = get_result.items.first().ok_or_else(|| {
            Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                "No content items found".into(),
            )))
        })?;

        if let Some(encoded_content) = &item.content {
            // GitHub API returns base64 encoded content with possible newlines
            let cleaned_encoded = encoded_content.replace("\n", "");

            // Decode the content
            let content = general_purpose::STANDARD
                .decode(&cleaned_encoded)
                .map_err(|e| {
                    Report::new(StorageError::GitHub(GitHubStorageError::EncodingError))
                        .attach_printable(format!("Failed to decode content: {}", e))
                })?;

            let content_str = String::from_utf8(content).map_err(|e| {
                Report::new(StorageError::GitHub(GitHubStorageError::EncodingError))
                    .attach_printable(format!("Failed to convert bytes to UTF-8: {}", e))
            })?;

            Ok(content_str)
        } else {
            Err(
                Report::new(StorageError::GitHub(GitHubStorageError::MissingData(
                    "No content found".into(),
                )))
                .attach_printable(format!("No content found for path: {}", path)),
            )
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use dotenv_codegen::dotenv;

//     fn init_gitbase() -> GitHubStorageResult<GitHubStorage> {
//         let token = dotenv!("GB_GITHUB_TOKEN");
//         let owner = dotenv!("GB_GITHUB_OWNER");
//         let repo = dotenv!("GB_GITHUB_REPO");

//         GitHubStorage::new(token, owner, repo, Some("main"))
//     }

//     #[tokio::test]
//     async fn test_write_file() {
//         let gitbase = init_gitbase().expect("Failed to initialize GitHubStorage");

//         let content = "Hello, world!";
//         let result = gitbase.write("test.txt", content).await;
//         match result {
//             Ok(meta) => println!("meta: {:?}", meta),
//             Err(e) => panic!("Error writing file: {}", e),
//         }
//     }

//     #[tokio::test]
//     async fn test_read_file() {
//         let gitbase = init_gitbase().expect("Failed to initialize GitHubStorage");

//         let result = gitbase.read("test.txt").await;
//         match result {
//             Ok(content) => println!("content: {}", content),
//             Err(e) => panic!("Error reading file: {}", e),
//         }
//     }
// }
