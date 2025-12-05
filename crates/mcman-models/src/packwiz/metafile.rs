use mcman_core::{Side, hash::HashFormat};
use serde::{Deserialize, Serialize};

/// *.pw.toml
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
#[serde(default)]
pub struct MetadataManifest {
    pub name: String,
    pub filename: String,
    pub download: ModDownload,
    pub option: ModOption,
    pub side: Side,
    pub update: Option<ModUpdate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ModDownload {
    pub url: Option<String>,
    pub hash: String,
    pub hash_format: HashFormat,
    #[serde(default)]
    pub mode: DownloadMode,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum DownloadMode {
    #[default]
    #[serde(alias = "")]
    Url,
    #[serde(rename = "metadata:curseforge")]
    Curseforge,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ModUpdate {
    #[serde(rename_all = "kebab-case")]
    Modrinth {
        mod_id: String,
        version: String,
    },
    #[serde(rename_all = "kebab-case")]
    Curseforge {
        file_id: u64,
        project_id: u64,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct ModOption {
    pub optional: bool,
    pub default: bool,
    pub description: Option<String>,
}
