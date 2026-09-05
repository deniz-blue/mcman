use std::path::{Path, PathBuf};

use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct PackageArtifact {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: Option<PathBuf>,
}

impl PackageArtifact {
    pub fn destination(&self) -> &Path {
        match &self.to {
            Some(to) => to,
            None => Path::new(
                self.from
                    .file_name()
                    .expect("a source with no file name is rejected when the package parses"),
            ),
        }
    }
}
