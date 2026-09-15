use std::path::{Path, PathBuf};

use crate::core::kdl::{KdlRead, Reader};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PackageArtifact {
    pub from: PathBuf,
    pub to: Option<PathBuf>,
}

impl KdlRead for PackageArtifact {
    fn read(reader: &mut Reader) -> Self {
        Self {
            from: reader.required_path_argument("source path"),
            to: reader.path_argument(),
        }
    }
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
