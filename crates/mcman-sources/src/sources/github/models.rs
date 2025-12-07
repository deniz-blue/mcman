use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubAsset {
    pub url: String,
    pub name: String,
    pub label: Option<String>,
    pub size: u64,
    pub download_count: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubRepository {
    pub description: Option<String>,
}