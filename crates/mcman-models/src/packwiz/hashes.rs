use std::ops::Deref;

use mcman_core::hash::Hashes;
use serde::{Serialize, Serializer, ser::SerializeMap};

pub struct PackwizHashes<'a>(pub &'a Hashes);

impl Deref for PackwizHashes<'_> {
    type Target = Hashes;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Serialize for PackwizHashes<'_> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = s.serialize_map(Some(self.0 .0.len()))?;
        for (hash_format, value) in &self.0 .0 {
            map.serialize_entry(hash_format.as_str_packwiz(), value)?;
        }
        map.end()
    }
}
