use std::{path::PathBuf, str::FromStr};

use miette::{IntoDiagnostic, Result};
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::ctx::Context;

/// Represents a multi-file structure either in the filesystem or on an URL
pub enum Location {
    Local(PathBuf),
    Remote(Url),
}

impl FromStr for Location {
    type Err = miette::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(url) = Url::parse(s) {
            return Ok(Location::Remote(url));
        }

        Ok(Location::Local(PathBuf::from(s)))
    }
}

impl Location {
    pub async fn read(&self, ctx: &Context, path: &str) -> Result<String> {
        match self {
            Location::Local(root) => {
                tokio::fs::read_to_string(root.join(path)).await.into_diagnostic()
            },
            Location::Remote(url) => {
                ctx.fetch_text(url.join(path).into_diagnostic()?).await
            }
        }
    }

    pub async fn read_json<T: DeserializeOwned>(&self, ctx: &Context, path: &str) -> Result<T> {
        let raw = self.read(ctx, path).await?;
        serde_json::from_str(&raw).into_diagnostic()
    }

    pub async fn read_toml<T: DeserializeOwned>(&self, ctx: &Context, path: &str) -> Result<T> {
        let raw = self.read(ctx, path).await?;
        toml::from_str(&raw).into_diagnostic()
    }
}


