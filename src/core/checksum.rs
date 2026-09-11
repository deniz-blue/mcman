use std::collections::BTreeMap;

use digest::DynDigest;

use crate::core::kdl::Reader;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn hasher(self) -> Box<dyn DynDigest> {
        match self {
            Self::Sha1 => Box::<sha1::Sha1>::default(),
            Self::Sha256 => Box::<sha2::Sha256>::default(),
            Self::Sha512 => Box::<sha2::Sha512>::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

            checksums.insert(algorithm, digest.to_lowercase());
        }

        checksums
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
}
