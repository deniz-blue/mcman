use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::providers::ProviderError;

#[derive(Debug, Error, Diagnostic)]
#[error("could not resolve `{addon}`")]
#[diagnostic(code(mcman::resolve))]
pub struct ResolveError {
    pub addon: String,
    #[label("declared here")]
    pub at: SourceSpan,
    #[source]
    #[diagnostic_source]
    pub source: ProviderError,
}
