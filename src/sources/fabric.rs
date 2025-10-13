use std::{borrow::Cow, collections::HashMap};

use crate::app::{App, CacheStrategy, ResolvedFile};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const FABRIC_META_URL: &str = "https://meta.fabricmc.net";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FabricLoader {
    pub separator: String,
    pub build: u64,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FabricInstaller {
    pub url: String,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

pub async fn fetch_loaders(client: &reqwest::Client) -> Result<Vec<FabricLoader>> {
    Ok(client
        .get(FABRIC_META_URL.to_owned() + "/v2/versions/loader")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub async fn fetch_installers(client: &reqwest::Client) -> Result<Vec<FabricInstaller>> {
    Ok(client
        .get(FABRIC_META_URL.to_owned() + "/v2/versions/installer")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub struct FabricAPI<'a>(pub &'a App);

impl FabricAPI<'_> {
    pub async fn fetch_loaders(&self) -> Result<Vec<FabricLoader>> {
        Ok(fetch_loaders(&self.0.http_client).await?)
    }

    pub async fn fetch_latest_loader(&self) -> Result<String> {
        Ok(self
            .fetch_loaders()
            .await?
            .first()
            .ok_or(anyhow!("No fabric loaders???"))?
            .version
            .clone())
    }

    pub async fn fetch_installers(&self) -> Result<Vec<FabricInstaller>> {
        Ok(fetch_installers(&self.0.http_client).await?)
    }

    pub async fn fetch_latest_installer(&self) -> Result<String> {
        Ok(self
            .fetch_installers()
            .await?
            .first()
            .ok_or(anyhow!("No fabric installers???"))?
            .version
            .clone())
    }

    pub async fn resolve_source(&self, loader: &str, installer: &str) -> Result<ResolvedFile> {
        let loader = match loader {
            "latest" => self.fetch_latest_loader().await?,
            id => id.to_owned(),
        };

        let installer = match installer {
            "latest" => self.fetch_latest_installer().await?,
            id => id.to_owned(),
        };

        let cached_file_path = format!(
            "fabric-server-{}-{installer}-{loader}.jar",
            self.0.mc_version()
        );

        Ok(ResolvedFile {
            url: format!(
                "{FABRIC_META_URL}/v2/versions/loader/{}/{loader}/{installer}/server/jar",
                self.0.mc_version()
            ),
            filename: cached_file_path.clone(),
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed("fabric"),
                path: cached_file_path,
            },
            size: None,
            hashes: HashMap::new(),
        })
    }
}
