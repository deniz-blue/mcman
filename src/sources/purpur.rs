use std::{borrow::Cow, collections::BTreeMap};

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::app::{App, CacheStrategy, ResolvedFile};

pub struct PurpurAPI<'a>(pub &'a App);

pub const API_URL: &str = "https://api.purpurmc.org/v2/purpur";
pub const CACHE_DIR: &str = "purpur";

impl PurpurAPI<'_> {
    pub async fn fetch_api<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response: T = self
            .0
            .http_client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response)
    }

    #[allow(unused)]
    pub async fn fetch_versions(&self) -> Result<Vec<String>> {
        Ok(self.fetch_api::<PurpurMCResponse>(API_URL).await?.versions)
    }

    pub async fn fetch_builds(&self, version: &str) -> Result<PurpurMCBuilds> {
        Ok(self
            .fetch_api::<PurpurMCVersion>(&format!("{API_URL}/{version}?detailed=true"))
            .await?
            .builds)
    }

    pub async fn fetch_build(&self, version: &str, build: &str) -> Result<PurpurMCBuild> {
        let builds = self.fetch_builds(version).await?;

        Ok(match build {
            "latest" => builds.latest.clone(),
            id => builds
                .all
                .iter()
                .find(|b| b.build == id)
                .ok_or(anyhow!("Cant find build '{id}' of Purpur {version}"))?
                .clone(),
        })
    }

    pub async fn resolve_source(&self, version: &str, build: &str) -> Result<ResolvedFile> {
        let resolved_build = self.fetch_build(version, build).await?;

        let cached_file_path = format!("purpur-{version}-{}.jar", resolved_build.build);

        Ok(ResolvedFile {
            url: format!("{API_URL}/{version}/{}/download", resolved_build.build),
            filename: cached_file_path.clone(),
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed(CACHE_DIR),
                path: cached_file_path,
            },
            size: None,
            hashes: BTreeMap::from([("md5".to_owned(), resolved_build.md5)]),
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PurpurMCResponse {
    pub project: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PurpurMCVersion {
    pub builds: PurpurMCBuilds,
    pub project: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PurpurMCBuilds {
    pub latest: PurpurMCBuild,
    pub all: Vec<PurpurMCBuild>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PurpurMCBuild {
    pub project: String,
    pub version: String,
    pub build: String,
    pub md5: String,
}
