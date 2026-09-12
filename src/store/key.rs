use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::store::error::{io, StoreError};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectKey(pub blake3::Hash);

impl ObjectKey {
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }

    pub fn from_hex(hex: &str) -> Result<Self, StoreError> {
        blake3::Hash::from_hex(hex)
            .map(ObjectKey)
            .map_err(|_| StoreError::NotAHash {
                hex: hex.to_owned(),
            })
    }

    pub async fn of_file(path: &Path) -> Result<Self, StoreError> {
        let source = path.to_owned();
        let hash = tokio::task::spawn_blocking(move || {
            let mut hasher = blake3::Hasher::new();
            let opened = std::fs::File::open(&source)?;
            hasher.update_reader(opened)?;
            Ok::<_, std::io::Error>(hasher.finalize())
        })
        .await
        .expect("hashing should not panic")
        .map_err(io("could not read", path))?;

        Ok(ObjectKey(hash))
    }
}

impl From<blake3::Hash> for ObjectKey {
    fn from(hash: blake3::Hash) -> Self {
        ObjectKey(hash)
    }
}

impl Serialize for ObjectKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ObjectKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        ObjectKey::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}
