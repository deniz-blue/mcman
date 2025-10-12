use anyhow::{anyhow, Result};

use crate::app::{App, ResolvedFile};

pub struct QuiltAPI<'a>(pub &'a App);

pub const QUILT_MAVEN_URL: &str = "https://maven.quiltmc.org/repository/release";
pub const QUILT_MAVEN_GROUP: &str = "org.quiltmc";
pub const QUILT_MAVEN_ARTIFACT: &str = "quilt-installer";
pub const QUILT_MAVEN_FILE: &str = "${artifact}-${version}.jar";

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
        "latest" => mcapi::quilt::fetch_loaders(client)
            .await?
            .iter()
            .find(|l| !l.version.contains("beta"))
            .ok_or(anyhow!("cant find latest loader version - None"))?
            .version
            .clone(),
        "latest-beta" => mcapi::quilt::fetch_loaders(client)
            .await?
            .first()
            .ok_or(anyhow!("cant find latest loader version - None"))?
            .version
            .clone(),
        id => id.to_owned(),
    })
}
