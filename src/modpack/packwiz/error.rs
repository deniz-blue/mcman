use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::core::{checksum::ChecksumMismatch, location::LocationError};

#[derive(Debug, Error, Diagnostic)]
pub enum PackwizError {
    #[error("`{}` escapes the pack", .path.display())]
    #[diagnostic(code(mcman::packwiz_escaping_path))]
    EscapingPath { path: PathBuf },

    #[error(transparent)]
    #[diagnostic(transparent)]
    Location(#[from] Box<LocationError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Checksum(#[from] ChecksumMismatch),
}

impl From<LocationError> for PackwizError {
    fn from(error: LocationError) -> Self {
        Self::Location(Box::new(error))
    }
}
