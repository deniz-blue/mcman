use std::path::PathBuf;

use kdl::KdlNode;

use crate::core::kdl::{Errors, Reader, Spanned};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyFile {
    pub from: PathBuf,
    pub to: PathBuf,
    pub overwrite: bool,
}

impl CopyFile {
    pub(super) fn read(node: &KdlNode, errors: &mut Errors) -> Spanned<Self> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let from = reader.required_path_argument("source path");
        let to = reader.required_path_argument("destination path");
        let overwrite = reader.flag_property("overwrite");
        reader.reject_unread();

        Spanned::new(
            Self {
                from,
                to,
                overwrite,
            },
            span,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymlinkFile {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl SymlinkFile {
    pub(super) fn read(node: &KdlNode, errors: &mut Errors) -> Spanned<Self> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let from = reader.required_path_argument("source path");
        let to = reader.required_path_argument("destination path");
        reader.reject_unread();

        Spanned::new(Self { from, to }, span)
    }
}
