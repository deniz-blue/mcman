use kdl::KdlNode;

use crate::{
    core::kdl::Errors,
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
            "git" => Self::Git(Git::read(node, errors)),
            _ => Self::Download(Download::read(node, errors)),
        }
    }
}
