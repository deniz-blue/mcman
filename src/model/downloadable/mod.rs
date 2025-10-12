use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::app::{App, CacheStrategy, Resolvable, ResolvedFile};

mod markdown;
mod meta;

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Downloadable {
    // sources
    Url {
        url: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "crate::util::is_default")]
        filename: Option<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "crate::util::is_default")]
        desc: Option<String>,
    },

    #[serde(alias = "mr")]
    Modrinth {
        id: String,
        #[serde(default = "latest")]
        version: String,
    },

    #[serde(alias = "cr")]
    CurseRinth {
        id: String,
        #[serde(default = "latest")]
        version: String,
    },

    Spigot {
        id: String,
        #[serde(default = "latest")]
        version: String,
    },

    Hangar {
        id: String,
        version: String,
    },

    #[serde(rename = "ghrel")]
    GithubRelease {
        repo: String,
        tag: String,
        asset: String,
    },

    // pain in the a-
    Jenkins {
        url: String,
        job: String,
        #[serde(default = "latest")]
        build: String,
        #[serde(default = "first")]
        artifact: String,
    },

    Maven {
        url: String,
        group: String,
        artifact: String,
        #[serde(default = "latest")]
        version: String,
        #[serde(default = "artifact")]
        filename: String,
    },
}

pub fn latest() -> String {
    "latest".to_owned()
}

pub fn first() -> String {
    "first".to_owned()
}

pub fn artifact() -> String {
    "artifact".to_owned()
}

impl Resolvable for Downloadable {
    async fn resolve_source(&self, app: &App) -> Result<ResolvedFile> {
        match self {
            Self::Url { url, filename, .. } => Ok(ResolvedFile {
                url: url.clone(),
                filename: if let Some(filename) = filename {
                    filename.clone()
                } else {
                    let url_clean = url.split('?').next().unwrap_or(url);
                    url_clean.split('/').next_back().unwrap().to_string()
                },
                cache: CacheStrategy::None,
                size: None,
                hashes: HashMap::new(),
            }),
            Self::Modrinth { id, version } => app.modrinth().resolve_source(id, version).await,
            Self::CurseRinth { id, version } => app.curserinth().resolve_source(id, version).await,
            Self::Spigot { id, version } => app.spigot().resolve_source(id, version).await,
            Self::Hangar { id, version } => app.hangar().resolve_source(id, version).await,
            Self::GithubRelease { repo, tag, asset } => {
                app.github().resolve_source(repo, tag, asset).await
            }
            Self::Jenkins {
                url,
                job,
                build,
                artifact,
            } => {
                app.jenkins()
                    .resolve_source(url, job, build, artifact)
                    .await
            }
            Self::Maven {
                url,
                group,
                artifact,
                version,
                filename,
            } => {
                app.maven()
                    .resolve_source(url, group, artifact, version, filename)
                    .await
            }
        }
    }
}
