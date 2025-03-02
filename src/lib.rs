#![allow(dead_code)]

use octocrab::Octocrab;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GitBase {
    client: Octocrab,
    owner: String,
    repo: String,
    cache: HashMap<String, Value>, // 简单缓存
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
