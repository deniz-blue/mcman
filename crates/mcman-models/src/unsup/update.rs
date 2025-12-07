use mcman_core::Side;
use serde::{Deserialize, Serialize};
use crate::unsup::UnsupHashFormat;

/// `"unsup_manifest": "update-1"`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateManifest {
    pub hash_function: UnsupHashFormat,
    pub changes: Vec<Change>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Change {
    pub path: String,
    pub from_size: Option<u128>,
    pub from_hash: Option<String>,
    pub to_size: u128,
    pub to_hash: Option<String>,
    pub url: Option<String>,
    pub envs: Vec<Side>,
    pub flavors: Vec<String>,
}
