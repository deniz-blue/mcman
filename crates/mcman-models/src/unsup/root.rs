use std::convert::Infallible;
use std::str::FromStr;
use mcman_core::Side;
use serde::{Deserialize, Serialize};
use crate::unsup::VersionInfo;

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
