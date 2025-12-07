use mcman_core::Side;

use serde::{Deserialize, Serialize};

use crate::unsup::{Change, UnsupHashFormat, VersionInfo};

/// `"unsup_manifest": "bootstrap-1"`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BootstrapManifest {
    pub version: VersionInfo,
    pub hash_function: UnsupHashFormat,
    pub files: Vec<Change>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BootstrapFile {
    pub path: String,
    pub size: u128,
    pub hash: Option<String>,
    pub url: Option<String>,
    pub envs: Vec<Side>,
    pub flavors: Vec<String>,
}
