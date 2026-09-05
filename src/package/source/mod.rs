use knus::Decode;

use crate::package::source::{download::Download, git::Git};

pub mod download;
pub mod git;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
#[knus(span_type = knus::span::Span)]
pub enum PackageSource {
    Download(Download),
    Git(Git),
}
