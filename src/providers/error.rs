use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ProviderError {
    #[error("a platform is needed to pick a compatible version")]
    #[diagnostic(
        code(mcman::platform_required),
        help("Declare a `platform` in this group or one above it.")
    )]
    PlatformRequired,

    #[error("no version matches `{requested}`")]
    #[diagnostic(code(mcman::no_matching_version))]
    NoMatchingVersion { requested: String },

    #[error("the chosen version has no file named `{file}`")]
    #[diagnostic(code(mcman::no_such_file))]
    NoSuchFile { file: String },

    #[error("`{type_name}` addons cannot be resolved yet")]
    #[diagnostic(code(mcman::unsupported_provider))]
    Unsupported { type_name: &'static str },

    #[error(transparent)]
    #[diagnostic(code(mcman::provider_request))]
    Request(Box<dyn std::error::Error + Send + Sync>),
}
