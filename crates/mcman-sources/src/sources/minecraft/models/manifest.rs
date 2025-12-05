use serde::{Deserialize, Serialize};

use crate::sources::minecraft::models::VersionType;

/// The version manifest, from piston-meta
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionIndex>,
}

impl VersionManifest {
    /// Find the version with id from the list
    #[must_use]
    pub fn find(&self, id: &str) -> Option<VersionIndex> {
        self.versions.iter().find(|v| v.id == id).cloned()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

/// The version info from a manifest's versions list
/// Use [`Self::fetch()`] to get an [`VersionInfo`] which contains more info about the version
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionIndex {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u8,
}
