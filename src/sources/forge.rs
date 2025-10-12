use anyhow::{anyhow, Context, Result};

use crate::app::{App, ResolvedFile};

pub static FORGE_MAVEN: &str = "https://maven.minecraftforge.net";
pub static FORGE_GROUP: &str = "net.minecraftforge";
pub static FORGE_ARTIFACT: &str = "forge";
pub static FORGE_FILENAME: &str = "${artifact}-${version}-installer.jar";

pub struct ForgeAPI<'a>(pub &'a App);

impl ForgeAPI<'_> {
    /// Returns a string list of filtered versions, everything after the first dash (-)
    pub async fn fetch_versions(&self) -> Result<Vec<String>> {
        let (_, versions) = self
            .0
            .maven()
            .fetch_versions(FORGE_MAVEN, FORGE_GROUP, FORGE_ARTIFACT)
            .await?;

        Ok(versions
            .iter()
            .filter_map(|s| {
                let v = s.split('-').collect::<Vec<_>>();

                if v[0] == self.0.mc_version() {
                    Some(v[1].to_owned())
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn fetch_latest(&self) -> Result<String> {
        crate::util::get_latest_semver(&self.fetch_versions().await?).ok_or(anyhow!(
            "No forge loader versions for {}",
            self.0.mc_version()
        ))
    }

    pub async fn resolve_version(&self, loader: &str) -> Result<String> {
        Ok(if loader == "latest" || loader.is_empty() {
            self.fetch_latest()
                .await
                .context("Getting latest Forge version")?
        } else {
            loader.to_owned()
        })
    }

    pub async fn resolve_source(&self, loader: &str) -> Result<ResolvedFile> {
        self.0
            .maven()
            .resolve_source(
                FORGE_MAVEN,
                FORGE_GROUP,
                FORGE_ARTIFACT,
                &format!(
                    "{}-{}",
                    self.0.mc_version(),
                    self.resolve_version(loader).await?
                ),
                FORGE_FILENAME,
            )
            .await
    }
}
