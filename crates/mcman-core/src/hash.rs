use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum HashFormat {
    Sha256,
    Sha512,
    Sha1,
    Md5,
    #[serde(rename = "murmur2")]
    #[default]
    Curseforge,
}



