use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "action")]
pub enum PistonRule {
    Allow(PistonRuleConstraints),
    Disallow(PistonRuleConstraints),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PistonRuleConstraints {
    pub os: Option<PistonOs>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PistonOs {
    pub name: String,
    pub arch: String,
    pub version: String,
}
