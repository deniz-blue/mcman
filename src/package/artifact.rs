use std::path::PathBuf;

use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PackageArtifact {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: Option<PathBuf>,
}
