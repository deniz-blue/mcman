use anyhow::{anyhow, Context, Result};
use semver::Version;

use crate::{app::App, app::ResolvedFile, util};

pub static NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases";
pub static NEOFORGE_GROUP: &str = "net.neoforged";
pub static NEOFORGE_OLD_ARTIFACT: &str = "forge";
pub static NEOFORGE_NEW_ARTIFACT: &str = "neoforge";
pub static NEOFORGE_FILENAME: &str = "${artifact}-${version}-installer.jar";

/// The first NeoForge version that breaks Forge compatibility
/// and has a different artifact name (1.20.2).
///
/// Source: https://www.reddit.com/r/feedthebeast/comments/17kitw5/poll_now_that_neoforge_and_minecraft_forge_are_no/
pub static NEOFORGE_BREAKOFF_VERSION: Version = Version::new(1, 20, 2);

pub struct NeoforgeAPI<'a>(pub &'a App);

impl<'a> NeoforgeAPI<'a> {
    pub async fn fetch_versions(&self) -> Result<Vec<String>> {
        let (_, versions) = self
            .0
            .maven()
            .fetch_versions(NEOFORGE_MAVEN, NEOFORGE_GROUP, self.get_artifact_id()?)
            .await?;

        if self.is_after_breakoff()? {
            // Post-breakoff version format: "mc_minor.mc_patch.loader_ver(-beta)"
            // e.g. "21.9.0-beta" or "21.3.93"
            let mc_ver = self.0.mc_version();
            let mc_trimmed = mc_ver.strip_prefix("1.").unwrap_or(mc_ver);

            Ok(versions
                .into_iter()
                .filter(|v| !v.contains("beta"))
                .filter(|v| v.starts_with(mc_trimmed))
                .collect())
        } else {
            Ok(versions
                .iter()
                .filter_map(|version| {
                    // Pre-breakoff version format: "mc_ver-loader_ver"
                    // e.g. "1.20.1-47.1.7"
                    let (mc_ver, loader_ver) = version.split_once('-')?;

                    if mc_ver == self.0.mc_version() {
                        Some(loader_ver.to_owned())
                    } else {
                        None
                    }
                })
                .collect())
        }
    }

    pub async fn fetch_latest(&self) -> Result<String> {
        util::get_latest_semver(&self.fetch_versions().await?).ok_or(anyhow!(
            "No NeoForge loader versions for {}",
            self.0.mc_version()
        ))
    }

    pub async fn resolve_version(&self, loader: &str) -> Result<String> {
        Ok(if loader == "latest" || loader.is_empty() {
            self.fetch_latest()
                .await
                .context("Getting latest NeoForge version")?
        } else {
            loader.to_owned()
        })
    }

    pub async fn resolve_source(&self, loader: &str) -> Result<ResolvedFile> {
        let version = if self.is_after_breakoff()? {
            self.resolve_version(loader).await?
        } else {
            format!(
                "{}-{}",
                self.0.mc_version(),
                self.resolve_version(loader).await?
            )
        };

        self.0
            .maven()
            .resolve_source(
                NEOFORGE_MAVEN,
                NEOFORGE_GROUP,
                self.get_artifact_id()?,
                &version,
                NEOFORGE_FILENAME,
            )
            .await
    }

    fn is_after_breakoff(&self) -> Result<bool> {
        let mc_version = Version::parse(self.0.mc_version())
            .context("Parsing mc version to determine neoforge version")?;

        Ok(mc_version >= NEOFORGE_BREAKOFF_VERSION)
    }

    fn get_artifact_id(&self) -> Result<&'static str> {
        let use_new_artifact = self.is_after_breakoff()?;

        if use_new_artifact {
            Ok(NEOFORGE_NEW_ARTIFACT)
        } else {
            Ok(NEOFORGE_OLD_ARTIFACT)
        }
    }
}
