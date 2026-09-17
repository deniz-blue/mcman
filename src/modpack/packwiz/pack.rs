use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::checksum::ChecksumAlgorithm;

pub const PACK_TOML: &str = "pack.toml";
pub const INDEX_TOML: &str = "index.toml";

// packwiz stamps every pack with the format version it was written for.
pub const PACK_FORMAT: &str = "packwiz:1.1.0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizPack {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub pack_format: String,
    pub index: PackwizFile,
    pub versions: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizIndex {
    pub hash_format: ChecksumAlgorithm,
    #[serde(default)]
    pub files: Vec<PackwizFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizFile {
    pub file: String,
    pub hash: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub metafile: bool,
}

fn is_false(flag: &bool) -> bool {
    !flag
}
