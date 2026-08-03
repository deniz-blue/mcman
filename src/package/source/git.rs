use std::path::PathBuf;

use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Git {
    pub url: String,
    pub path: Option<PathBuf>,
}
