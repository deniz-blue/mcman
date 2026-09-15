use kdl::KdlNode;

use crate::{
    core::kdl::{Errors, KdlRead, KdlWrite},
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
            "git" => Self::Git(Git::read_node(node, errors)),
            _ => Self::Download(Download::read_node(node, errors)),
        }
    }

    pub(crate) fn to_kdl(&self) -> KdlNode {
        let (name, source): (&str, &dyn KdlWrite) = match self {
            Self::Download(download) => ("download", download),
            Self::Git(git) => ("git", git),
        };

        source.to_kdl(name)
    }
}
