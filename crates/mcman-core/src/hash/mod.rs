use std::{collections::HashMap, ops::Deref};

use digest::DynDigest;
use serde::{Deserialize, Serialize};

mod murmur2;
mod format;
pub use murmur2::*;
pub use format::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct Hashes(pub HashMap<HashFormat, String>);

impl Hashes {
    pub fn new_single(format: HashFormat, hash: String) -> Self {
        let mut map = HashMap::new();
        map.insert(format, hash);
        Hashes(map)
    }

    pub fn get_best_hash(&self) -> Option<(&HashFormat, &String)> {
        static PREFERRED_ORDER: [HashFormat; 6] = [
            HashFormat::Sha512,
            HashFormat::Sha384,
            HashFormat::Sha256,
            HashFormat::Sha1,
            HashFormat::Md5,
            HashFormat::Murmur2,
        ];

        for format in &PREFERRED_ORDER {
            if let Some(hash) = self.0.get(format) {
                return Some((format, hash));
            }
        }

        None
    }

    pub fn get_best_hasher(&self) -> Option<Box<dyn DynDigest>> {
        if let Some((format, _)) = self.get_best_hash() {
            Some((*format).into())
        } else {
            None
        }
    }
}

impl Deref for Hashes {
    type Target = HashMap<HashFormat, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
