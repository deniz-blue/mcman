use std::path::{Path, PathBuf};

use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlWrite, Reader};

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

impl KdlWrite for PackageArtifact {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.from.display().to_string());

        if let Some(to) = &self.to {
            node.push(to.display().to_string());
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
