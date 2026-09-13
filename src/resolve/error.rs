use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::providers::ProviderError;

#[derive(Debug, Error, Diagnostic)]
#[error("could not resolve `{declaration}`")]
#[diagnostic(code(mcman::resolve))]
pub struct ResolveError {
    pub declaration: String,
    #[label("declared here")]
    pub at: SourceSpan,
    #[source]
    #[diagnostic_source]
    pub source: ProviderError,
}

impl ResolveError {
    pub fn of(declaration: &impl Display, at: SourceSpan, source: ProviderError) -> Self {
        Self {
            declaration: declaration.to_string(),
            at,
            source,
        }
    }
}
