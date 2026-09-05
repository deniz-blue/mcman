use std::path::PathBuf;

use kdl::KdlNode;

use crate::core::kdl::{Errors, Reader};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Git {
    pub url: String,
    pub path: Option<PathBuf>,
}

impl Git {
    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let url = reader.required_argument("repository url");
        let path = reader.path_property("path");
        reader.reject_unread();

        Self { url, path }
    }
}
