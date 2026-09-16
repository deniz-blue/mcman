use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::core::checksum::ChecksumAlgorithm;

pub const PACK_TOML: &str = "pack.toml";
pub const INDEX_TOML: &str = "index.toml";

// packwiz stamps every pack with the format version it was written for.
pub const PACK_FORMAT: &str = "packwiz:1.1.0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Pack {
    pub name: String,
    pub pack_format: String,
    pub index: PackFile,
    pub versions: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackIndex {
    pub hash_format: ChecksumAlgorithm,
    #[serde(default)]
    pub files: Vec<PackFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackFile {
    pub file: String,
    pub hash: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub metafile: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Metafile {
    pub name: String,
    pub filename: String,
    pub download: MetafileDownload,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetafileDownload {
    pub url: String,
    pub hash: String,
    pub hash_format: ChecksumAlgorithm,
}

// packwiz index paths are slash-separated whatever the host writes them on.
pub fn slashed(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn is_false(flag: &bool) -> bool {
    !flag
}
