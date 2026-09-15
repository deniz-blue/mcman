use std::path::PathBuf;

use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Git {
    pub url: String,
    pub path: Option<PathBuf>,
}

impl KdlRead for Git {
    fn read(reader: &mut Reader) -> Self {
        Self {
            url: reader.required_argument("repository url"),
            path: reader.path_property("path"),
        }
    }
}

impl KdlWrite for Git {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.url.as_str());
        if let Some(path) = &self.path {
            node.push(("path", path.display().to_string()));
        }
    }
}
