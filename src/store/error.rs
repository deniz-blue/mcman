use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum StoreError {
    #[error("`{}` is not a directory", .path.display())]
    NotADirectory { path: PathBuf },

    #[error("the store at `{}` is in use", .path.display())]
    #[diagnostic(help("Another mcman is building. Wait for it to finish, or run this later."))]
    InUse { path: PathBuf },

    #[error("`{hex}` is not a blake3 hash")]
    NotAHash { hex: String },

    #[error("{action} `{}`", .path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub(super) fn io(
    action: &'static str,
    path: impl Into<PathBuf>,
) -> impl FnOnce(std::io::Error) -> StoreError {
    let path = path.into();
    move |source| StoreError::Io {
        action,
        path,
        source,
    }
}
