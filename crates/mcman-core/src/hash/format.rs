use crate::hash::Murmur2;
use digest::DynDigest;
use serde::{Deserialize, Serialize};
use sha1::Digest;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum HashFormat {
    #[serde(alias = "sha-1")]
    Sha1,
    #[serde(alias = "sha-2 256")]
    #[default]
    Sha256,
    #[serde(alias = "sha-2 384")]
    Sha384,
    #[serde(alias = "sha-2 512")]
    Sha512,
    #[serde(alias = "sha-2 512/256")]
    #[serde(rename = "sha512/256")]
    Sha512_256,
    #[serde(alias = "md-5")]
    Md5,
    #[serde(alias = "curseforge")]
    #[serde(alias = "murmur2-cf")]
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
            HashFormat::Sha512_256 => Box::new(sha2::Sha512_256::new()),
        }
    }
}

impl HashFormat {
    pub fn as_str_packwiz(&self) -> &'static str {
        match self {
            HashFormat::Murmur2 => "murmur2",
            HashFormat::Md5 => "md5",
            HashFormat::Sha1 => "sha1",
            HashFormat::Sha256 => "sha256",
            HashFormat::Sha384 => "sha384",
            HashFormat::Sha512 => "sha512",
            HashFormat::Sha512_256 => "sha512-256",
        }
    }

    pub fn as_str_unsup(&self) -> &'static str {
        match self {
            HashFormat::Murmur2 => "Murmur2-CF",
            HashFormat::Md5 => "MD5",
            HashFormat::Sha1 => "SHA-1",
            HashFormat::Sha256 => "SHA-2 256",
            HashFormat::Sha384 => "SHA-2 384",
            HashFormat::Sha512 => "SHA-2 512",
            HashFormat::Sha512_256 => "SHA-2 512/256",
        }
    }
}
