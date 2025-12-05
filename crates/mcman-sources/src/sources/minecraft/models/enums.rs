use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    #[default]
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadType {
    Client,
    ClientMappings,
    Server,
    ServerMappings,
    WindowsServer,
}
