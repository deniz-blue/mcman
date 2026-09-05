use std::path::{Path, PathBuf};

use kdl::KdlNode;

use crate::core::kdl::{Errors, Reader};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PackageArtifact {
    pub from: PathBuf,
    pub to: Option<PathBuf>,
}

impl PackageArtifact {
    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let from = reader.required_path_argument("source path");
        let to = reader.path_argument();
        reader.reject_unread();

        Self { from, to }
    }

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
