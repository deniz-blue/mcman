use serde::{Deserialize, Serialize};

use crate::core::checksum::ChecksumAlgorithm;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizMetafile {
    pub name: String,
    pub filename: String,
    pub download: PackwizMetafileDownload,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizMetafileDownload {
    pub url: String,
    pub hash: String,
    pub hash_format: ChecksumAlgorithm,
}
