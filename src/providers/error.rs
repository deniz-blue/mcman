use miette::Diagnostic;
use thiserror::Error;

use crate::{
    core::{checksum::ChecksumMismatch, location::LocationError},
    modpack::mrpack::MrpackError,
};

#[derive(Debug, Error, Diagnostic)]
pub enum ProviderError {
    #[error("a platform is needed to pick a compatible version")]
    #[diagnostic(
        code(mcman::platform_required),
        help("Declare `platform minecraft version=\"…\"` and the server's platform in this group or one above it.")
    )]
    PlatformRequired,

    #[error("no version matches `{requested}`")]
    #[diagnostic(code(mcman::no_matching_version))]
    NoMatchingVersion { requested: String },

    #[error("the chosen version has no file named `{file}`")]
    #[diagnostic(code(mcman::no_such_file))]
    NoSuchFile { file: String },

    #[error("`{type_name}` cannot be resolved yet")]
    #[diagnostic(code(mcman::unsupported_provider))]
    Unsupported { type_name: &'static str },

    #[error(transparent)]
    #[diagnostic(code(mcman::provider_request))]
    Request(Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Location(#[from] Box<LocationError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Mrpack(#[from] Box<MrpackError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Checksum(#[from] ChecksumMismatch),
}

impl From<LocationError> for ProviderError {
    fn from(error: LocationError) -> Self {
        Self::Location(Box::new(error))
    }
}

impl From<MrpackError> for ProviderError {
    fn from(error: MrpackError) -> Self {
        Self::Mrpack(Box::new(error))
    }
}
