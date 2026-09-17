use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize, Serializer};

use crate::{
    core::checksum::Checksums,
    modpack::{slashed, Side},
    package::source::download::Download,
};

pub const INDEX_NAME: &str = "modrinth.index.json";
pub const FORMAT_VERSION: u32 = 1;
pub const GAME: &str = "minecraft";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub files: Vec<MrpackFile>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackFile {
    #[serde(serialize_with = "slashed_path")]
    pub path: PathBuf,
    pub hashes: Checksums,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<MrpackEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct MrpackEnv {
    pub client: MrpackSupport,
    pub server: MrpackSupport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MrpackSupport {
    Required,
    Optional,
    Unsupported,
    #[serde(other)]
    Unknown,
}

impl MrpackFile {
    pub fn supports(&self, side: Side) -> bool {
        let Some(env) = &self.env else {
            return true;
        };

        let support = match side {
            Side::Client => env.client,
            Side::Server => env.server,
        };

        support != MrpackSupport::Unsupported
    }

    pub fn to_download(&self) -> Download {
        Download {
            url: self.downloads.first().cloned().unwrap_or_default(),
            path: Some(self.path.clone()),
            checksums: self.hashes.clone(),
            size: Some(self.file_size),
        }
    }
}

fn slashed_path<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&slashed(path))
}
