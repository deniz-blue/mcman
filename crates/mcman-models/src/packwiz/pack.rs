use std::collections::HashMap;

use mcman_core::hash::HashFormat;
use serde::{Deserialize, Serialize};

pub static PACK_TOML: &str = "pack.toml";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PackManifest {
    pub name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub pack_format: String,
    pub index: FileEntry,
    pub versions: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct IndexManifest {
    pub hash_format: HashFormat,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct FileEntry {
    pub file: String,
    pub hash: String,
    pub hash_format: Option<String>,

    pub alias: Option<String>,
    #[serde(default)]
    pub metafile: bool,
    #[serde(default)]
    pub preserve: bool,
}
