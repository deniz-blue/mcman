use std::ops::Deref;

use mcman_core::hash::HashFormat;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct UnsupHashFormat(pub HashFormat);
impl serde::Serialize for UnsupHashFormat {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(self.0.as_str_unsup())
    }
}

impl Deref for UnsupHashFormat {
    type Target = HashFormat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
