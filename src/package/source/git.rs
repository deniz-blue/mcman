use std::path::PathBuf;

use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Git {
    #[knus(argument)]
    pub url: String,
    #[knus(property)]
    pub path: Option<PathBuf>,
}
