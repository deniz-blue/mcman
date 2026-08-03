use std::path::PathBuf;

use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PackageLink {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: Option<PathBuf>,
}
