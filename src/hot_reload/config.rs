use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use glob::Pattern;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(tag = "type", try_from = "String", into = "String")]
pub enum HotReloadAction {
    #[default]
    Reload,
    Restart,
    #[serde(alias = "run")]
    RunCommand(String),
}

impl TryFrom<String> for HotReloadAction {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        if value.starts_with('/') {
            Ok(Self::RunCommand(
                value.strip_prefix('/').unwrap().to_string(),
            ))
        } else {
            match value.to_lowercase().as_str() {
                "reload" => Ok(Self::Reload),
                "restart" => Ok(Self::Restart),
                _ => Err(anyhow!("Cant parse HotReloadAction: {value}")),
            }
        }
    }
}

impl From<HotReloadAction> for String {
    fn from(val: HotReloadAction) -> Self {
        match val {
            HotReloadAction::Reload => String::from("reload"),
            HotReloadAction::Restart => String::from("restart"),
            HotReloadAction::RunCommand(cmd) => format!("/{cmd}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HotReloadConfig {
    #[serde(skip)]
    pub path: PathBuf,

    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")]
    pub files: Vec<Entry>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub events: HashMap<String, HotReloadAction>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./hotreload.toml"),
            events: HashMap::new(),
            files: vec![Entry {
                path: Pattern::new("server.properties").unwrap(),
                action: HotReloadAction::Reload,
            }],
        }
    }
}

impl HotReloadConfig {
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let mut h: Self = toml::from_str(&data)?;
        h.path.clone_from(path);
        Ok(h)
    }

    pub fn save(&self) -> Result<()> {
        let cfg_str = toml::to_string_pretty(&self)?;
        let mut f = File::create(&self.path)?;
        f.write_all(cfg_str.as_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Entry {
    #[serde(with = "super::pattern_serde")]
    pub path: Pattern,
    pub action: HotReloadAction,
}
