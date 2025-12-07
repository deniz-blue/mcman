use std::{convert::Infallible, str::FromStr};

use mcman_core::{Side, hash::HashFormat};
use serde::{Deserialize, Serialize};

/// `"unsup_manifest": "root-1"`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RootManifest {
    pub name: String,
    pub versions: Versions,
    #[serde(default)]
    pub flavor_groups: Vec<FlavorGroup>,
    #[serde(default)]
    pub creator: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Versions {
    pub current: VersionInfo,
    #[serde(default)]
    pub history: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub code: u128,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FlavorGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub envs: Vec<Side>,
    pub description: Option<String>,
    pub choices: Vec<FlavorChoice>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FlavorChoice {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl FromStr for FlavorChoice {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            id: s.to_owned(),
            name: None,
            description: None,
        })
    }
}

/// `"unsup_manifest": "update-1"`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateManifest {
    pub hash_function: HashFormat,
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

/// `"unsup_manifest": "bootstrap-1"`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BootstrapManifest {
    pub version: VersionInfo,
    pub hash_function: HashFormat,
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
