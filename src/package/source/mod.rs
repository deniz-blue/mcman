use kdl::KdlNode;

use crate::{
    core::kdl::{read_node, Errors, KdlWrite},
    package::source::{download::Download, git::Git},
};

pub mod download;
pub mod git;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageSource {
    Download(Download),
    Git(Git),
}

impl PackageSource {
    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        match node.name().value() {
            "git" => Self::Git(read_node(node, errors)),
            _ => Self::Download(read_node(node, errors)),
        }
    }

    pub(crate) fn to_kdl(&self) -> KdlNode {
        let (name, source): (&str, &dyn KdlWrite) = match self {
            Self::Download(download) => ("download", download),
            Self::Git(git) => ("git", git),
        };

        let mut node = KdlNode::new(name);
        source.write(&mut node);

        node
    }
}
