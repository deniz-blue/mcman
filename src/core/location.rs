use std::{
    fmt::{Debug, Display},
    io,
    path::{Path, PathBuf},
    pin::Pin,
};

use miette::Diagnostic;
use reqwest::Url;
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;

use crate::core::AppContext;

#[derive(Clone, PartialEq, Eq)]
pub enum Location {
    Path(PathBuf),
    Url(Url),
}

impl Location {
    pub fn join(&self, reference: &str) -> Result<Self, LocationError> {
        if let Some(absolute) = Self::absolute(reference) {
            return Ok(absolute);
        }

        Ok(match self {
            Self::Path(path) => {
                let base = path.parent().unwrap_or_else(|| Path::new(""));
                Self::Path(base.join(reference))
            }
            Self::Url(url) => {
                let joined = url.join(reference).map_err(|source| LocationError::Join {
                    location: self.clone(),
                    reference: reference.to_owned(),
                    source,
                })?;
                Self::Url(joined)
            }
        })
    }

    fn absolute(text: &str) -> Option<Self> {
        match Url::parse(text) {
            Ok(url) if url.has_host() => Some(Self::Url(url)),
            _ => Path::new(text)
                .is_absolute()
                .then(|| Self::Path(PathBuf::from(text))),
        }
    }

    pub async fn read(
        &self,
        ctx: &AppContext,
    ) -> Result<Pin<Box<dyn AsyncRead + Send>>, LocationError> {
        match self {
            Self::Path(path) => {
                let file = tokio::fs::File::open(path)
                    .await
                    .map_err(|source| self.io(source))?;
                Ok(Box::pin(file))
            }
            Self::Url(url) => {
                let response = ctx
                    .cached_http
                    .get(url.clone())
                    .send()
                    .await
                    .map_err(|source| self.request(source))?
                    .error_for_status()
                    .map_err(|source| self.request(source.into()))?;
                let body = response
                    .bytes_stream()
                    .map(|chunk| chunk.map_err(io::Error::other));
                Ok(Box::pin(StreamReader::new(body)))
            }
        }
    }

    pub async fn read_bytes(&self, ctx: &AppContext) -> Result<Vec<u8>, LocationError> {
        let mut bytes = Vec::new();
        self.read(ctx)
            .await?
            .read_to_end(&mut bytes)
            .await
            .map_err(|source| self.io(source))?;
        Ok(bytes)
    }

    pub async fn read_json<T: DeserializeOwned>(
        &self,
        ctx: &AppContext,
    ) -> Result<T, LocationError> {
        let bytes = self.read_bytes(ctx).await?;
        serde_json::from_slice(&bytes).map_err(|source| LocationError::Json {
            location: self.clone(),
            source,
        })
    }

    pub async fn read_toml<T: DeserializeOwned>(
        &self,
        ctx: &AppContext,
    ) -> Result<T, LocationError> {
        let bytes = self.read_bytes(ctx).await?;
        let text = std::str::from_utf8(&bytes).map_err(|source| LocationError::Utf8 {
            location: self.clone(),
            source,
        })?;
        toml::from_str(text).map_err(|source| LocationError::Toml {
            location: self.clone(),
            source: Box::new(source),
        })
    }

    fn io(&self, source: io::Error) -> LocationError {
        LocationError::Io {
            location: self.clone(),
            source,
        }
    }

    fn request(&self, source: reqwest_middleware::Error) -> LocationError {
        LocationError::Request {
            location: self.clone(),
            source,
        }
    }
}

impl From<String> for Location {
    fn from(text: String) -> Self {
        match Url::parse(&text) {
            Ok(url) if url.has_host() => Self::Url(url),
            _ => Self::Path(PathBuf::from(text)),
        }
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => f.debug_tuple("Path").field(path).finish(),
            Self::Url(url) => f.debug_tuple("Url").field(&url.as_str()).finish(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => f.write_str(&path.display().to_string()),
            Self::Url(url) => f.write_str(url.as_str()),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum LocationError {
    #[error("`{reference}` does not sit next to `{location}`")]
    #[diagnostic(code(mcman::location_join))]
    Join {
        location: Location,
        reference: String,
        #[source]
        source: url::ParseError,
    },

    #[error("could not read `{location}`")]
    #[diagnostic(code(mcman::location_io))]
    Io {
        location: Location,
        #[source]
        source: io::Error,
    },

    #[error("could not fetch `{location}`")]
    #[diagnostic(code(mcman::location_request))]
    Request {
        location: Location,
        #[source]
        source: reqwest_middleware::Error,
    },

    #[error("`{location}` is not valid JSON")]
    #[diagnostic(code(mcman::location_json))]
    Json {
        location: Location,
        #[source]
        source: serde_json::Error,
    },

    #[error("`{location}` is not valid TOML")]
    #[diagnostic(code(mcman::location_toml))]
    Toml {
        location: Location,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("`{location}` is not valid UTF-8")]
    #[diagnostic(code(mcman::location_utf8))]
    Utf8 {
        location: Location,
        #[source]
        source: std::str::Utf8Error,
    },
}
