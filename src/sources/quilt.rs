use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::app::{App, ResolvedFile};

pub struct QuiltAPI<'a>(pub &'a App);

pub const QUILT_META_URL: &str = "https://meta.quiltmc.org";
pub const QUILT_MAVEN_URL: &str = "https://maven.quiltmc.org/repository/release";
pub const QUILT_MAVEN_GROUP: &str = "org.quiltmc";
pub const QUILT_MAVEN_ARTIFACT: &str = "quilt-installer";
pub const QUILT_MAVEN_FILE: &str = "${artifact}-${version}.jar";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuiltLoader {
    pub separator: String,
    pub build: u64,
    pub maven: String,
    pub version: String,
}

pub async fn fetch_loaders(client: &reqwest::Client) -> Result<Vec<QuiltLoader>> {
    let versions: Vec<QuiltLoader> = client
        .get(QUILT_META_URL.to_owned() + "/v3/versions/loader")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(versions)
}

impl QuiltAPI<'_> {
    pub async fn resolve_installer(&self, version: &str) -> Result<ResolvedFile> {
        self.0
            .maven()
            .resolve_source(
                QUILT_MAVEN_URL,
                QUILT_MAVEN_GROUP,
                QUILT_MAVEN_ARTIFACT,
                version,
                QUILT_MAVEN_FILE,
            )
            .await
    }
}

pub async fn map_quilt_loader_version(client: &reqwest::Client, loader: &str) -> Result<String> {
    Ok(match loader {
        "latest" => fetch_loaders(client)
            .await?
            .iter()
            .find(|l| !l.version.contains("beta"))
            .ok_or(anyhow!("cant find latest loader version - None"))?
            .version
            .clone(),
        "latest-beta" => fetch_loaders(client)
            .await?
            .first()
            .ok_or(anyhow!("cant find latest loader version - None"))?
            .version
            .clone(),
        id => id.to_owned(),
    })
}
