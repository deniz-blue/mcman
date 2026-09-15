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

    pub(crate) fn node_name(&self) -> &'static str {
        match self {
            Self::Download(_) => "download",
            Self::Git(_) => "git",
        }
    }
}

impl KdlWrite for PackageSource {
    fn write(&self, node: &mut KdlNode) {
        match self {
            Self::Download(download) => download.write(node),
            Self::Git(git) => git.write(node),
        }
    }
}
