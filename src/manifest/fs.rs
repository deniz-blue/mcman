use std::path::PathBuf;

use kdl::KdlNode;

use crate::core::kdl::{debug_without_span, Errors, Reader};

#[derive(Clone, PartialEq, Eq)]
pub struct CopyFile {
    pub from: PathBuf,
    pub to: PathBuf,
    pub overwrite: bool,
    pub span: miette::SourceSpan,
}

debug_without_span!(CopyFile {
    from,
    to,
    overwrite
});

impl CopyFile {
    pub(super) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let from = reader.required_path_argument("source path");
        let to = reader.required_path_argument("destination path");
        let overwrite = reader.flag_property("overwrite");
        reader.reject_unread();

        Self {
            from,
            to,
            overwrite,
            span,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SymlinkFile {
    pub from: PathBuf,
    pub to: PathBuf,
    pub span: miette::SourceSpan,
}

debug_without_span!(SymlinkFile { from, to });

impl SymlinkFile {
    pub(super) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let from = reader.required_path_argument("source path");
        let to = reader.required_path_argument("destination path");
        reader.reject_unread();

        Self { from, to, span }
    }
}
