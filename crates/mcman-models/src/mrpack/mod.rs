use std::collections::HashMap;

use mcman_core::{Side, hash::Hashes};
use serde::{Deserialize, Serialize};

pub const MRPACK_MIME_TYPE: &str = "application/x-modrinth-modpack+zip";
pub const MRPACK_EXT: &str = "mrpack";
pub const MRPACK_INDEX_FILENAME: &str = "modrinth.index.json";
pub const MRPACK_OVERRIDES: &str = "overrides";
pub const MRPACK_SERVER_OVERRIDES: &str = "server-overrides";
pub const MRPACK_CLIENT_OVERRIDES: &str = "client-overrides";

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MRPackIndex {
    pub game: String,
    pub name: String,
    pub version_id: String,
    pub summary: Option<String>,
    pub files: Vec<MRPackFile>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MRPackFile {
    path: String,
    hashes: Hashes,
    #[serde(default)]
    env: HashMap<Side, EnvSupport>,
    file_size: u64,
    downloads: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EnvSupport {
    Required,
    Optional,
    Unsupported,
}
