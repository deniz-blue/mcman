use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    core::checksum::{ChecksumAlgorithm, Checksums},
    package::source::download::Download,
};

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

impl PackwizMetafile {
    pub fn to_download(&self, path: &Path) -> Download {
        let mut checksums = Checksums::default();
        checksums.insert(self.download.hash_format, self.download.hash.clone());

        let directory = path.parent().unwrap_or_else(|| Path::new(""));

        Download {
            url: self.download.url.clone(),
            path: Some(directory.join(&self.filename)),
            checksums,
            size: None,
        }
    }
}
