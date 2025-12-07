use std::collections::HashMap;

use digest::DynDigest;
use serde::{Deserialize, Serialize};

mod murmur2;
pub use murmur2::*;
use sha1::Digest;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct Hashes(HashMap<HashFormat, String>);

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum HashFormat {
    #[serde(alias = "SHA-1")]
    Sha1,
    #[serde(alias = "SHA-2 256")]
    Sha256,
    #[serde(alias = "SHA-2 384")]
    Sha384,
    #[serde(alias = "SHA-2 512")]
    Sha512,
    Md5,
    #[serde(alias = "curseforge")]
    #[default]
    Murmur2,
}

impl Into<Box<dyn DynDigest>> for HashFormat {
    fn into(self) -> Box<dyn DynDigest> {
        match self {
            HashFormat::Murmur2 => Box::new(Murmur2::new()),
            HashFormat::Md5 => Box::new(md5::Md5::new()),
            HashFormat::Sha512 => Box::new(sha2::Sha512::new()),
            HashFormat::Sha1 => Box::new(sha1::Sha1::new()),
            HashFormat::Sha256 => Box::new(sha2::Sha256::new()),
            HashFormat::Sha384 => Box::new(sha2::Sha384::new()),
        }
    }
}
