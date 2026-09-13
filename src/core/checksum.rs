use std::collections::BTreeMap;

use digest::DynDigest;
use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;

use crate::core::kdl::Reader;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl ChecksumAlgorithm {
    pub const ALL: [Self; 3] = [Self::Sha1, Self::Sha256, Self::Sha512];

    pub fn name(self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    pub fn digest_length(self) -> usize {
        match self {
            Self::Sha1 => 40,
            Self::Sha256 => 64,
            Self::Sha512 => 128,
        }
    }

    pub fn digest(self, bytes: &[u8]) -> String {
        let mut hasher = self.hasher();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    pub fn hasher(self) -> Box<dyn DynDigest> {
        match self {
            Self::Sha1 => Box::<sha1::Sha1>::default(),
            Self::Sha256 => Box::<sha2::Sha256>::default(),
            Self::Sha512 => Box::<sha2::Sha512>::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Checksums(BTreeMap<ChecksumAlgorithm, String>);

impl Checksums {
    pub fn read(reader: &mut Reader) -> Self {
        let mut checksums = Self::default();

        for algorithm in ChecksumAlgorithm::ALL {
            let Some(digest) = reader.property(algorithm.name()) else {
                continue;
            };

            if digest.len() != algorithm.digest_length() {
                reader.reject(format!(
                    "`{}` is {} hex characters, found {}",
                    algorithm.name(),
                    algorithm.digest_length(),
                    digest.len()
                ));
                continue;
            }

            checksums.insert(algorithm, digest);
        }

        checksums
    }

    pub fn verify(&self, bytes: &[u8]) -> Result<(), ChecksumMismatch> {
        let Some((algorithm, expected)) = self.strongest() else {
            return Ok(());
        };

        let found = algorithm.digest(bytes);
        if found.eq_ignore_ascii_case(expected) {
            return Ok(());
        }

        Err(ChecksumMismatch {
            algorithm,
            expected: expected.to_owned(),
            found,
        })
    }

    pub fn insert(&mut self, algorithm: ChecksumAlgorithm, digest: String) {
        self.0.insert(algorithm, digest);
    }

    pub fn get(&self, algorithm: ChecksumAlgorithm) -> Option<&str> {
        self.0.get(&algorithm).map(String::as_str)
    }

    pub fn strongest(&self) -> Option<(ChecksumAlgorithm, &str)> {
        self.0
            .last_key_value()
            .map(|(algorithm, digest)| (*algorithm, digest.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (ChecksumAlgorithm, &str)> {
        self.0
            .iter()
            .map(|(algorithm, digest)| (*algorithm, digest.as_str()))
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("has {} {found}, expected {expected}", .algorithm.name())]
#[diagnostic(code(mcman::checksum_mismatch))]
pub struct ChecksumMismatch {
    pub algorithm: ChecksumAlgorithm,
    pub expected: String,
    pub found: String,
}
