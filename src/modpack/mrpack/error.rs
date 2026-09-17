use std::{io, path::PathBuf};

use miette::Diagnostic;
use thiserror::Error;
use zip::result::ZipError;

use crate::modpack::mrpack::index::{FORMAT_VERSION, GAME, INDEX_NAME};

#[derive(Debug, Error, Diagnostic)]
pub enum MrpackError {
    #[error("the pack has no `{INDEX_NAME}`")]
    #[diagnostic(code(mcman::mrpack_no_index))]
    NoIndex,

    #[error("the pack index could not be read")]
    #[diagnostic(code(mcman::mrpack_read_index))]
    ReadIndex(#[source] serde_json::Error),

    #[error("the pack index could not be written")]
    #[diagnostic(code(mcman::mrpack_write_index))]
    WriteIndex(#[source] serde_json::Error),

    #[error("the pack uses format version {version}, only {FORMAT_VERSION} is supported")]
    #[diagnostic(code(mcman::mrpack_format))]
    UnsupportedFormat { version: u32 },

    #[error("the pack is for `{game}`, not {GAME}")]
    #[diagnostic(code(mcman::mrpack_game))]
    NotMinecraft { game: String },

    #[error("`{}` escapes the instance directory", .path.display())]
    #[diagnostic(code(mcman::mrpack_escaping_path))]
    EscapingPath { path: PathBuf },

    #[error("`{}` has no download url", .path.display())]
    #[diagnostic(code(mcman::mrpack_no_downloads))]
    NoDownloads { path: PathBuf },

    #[error("could not read `{name}` from the pack")]
    #[diagnostic(code(mcman::mrpack_read))]
    Read {
        name: String,
        #[source]
        source: io::Error,
    },

    #[error("could not write `{name}` into the pack")]
    #[diagnostic(code(mcman::mrpack_write))]
    Write {
        name: String,
        #[source]
        source: io::Error,
    },

    #[error(transparent)]
    #[diagnostic(code(mcman::mrpack_zip))]
    Zip(#[from] ZipError),
}
