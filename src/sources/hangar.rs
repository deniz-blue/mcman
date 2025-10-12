use std::{borrow::Cow, collections::HashMap};

use anyhow::{anyhow, Context, Result};
use mcapi::hangar::{Platform, ProjectVersion};

use crate::{
    app::{App, CacheStrategy, ResolvedFile},
    model::ServerType,
};

pub async fn get_project_version(
    http_client: &reqwest::Client,
    id: &str,
    filter: Option<mcapi::hangar::PlatformFilter>,
    platform_version: Option<String>,
    plugin_version: Option<&str>,
) -> Result<mcapi::hangar::ProjectVersion> {
    // Use the provided filter or create a default one.
    let mut current_filter = filter.unwrap_or_default();

    // Closure to search for a version in a page.
    let find_version =
        |versions: &[mcapi::hangar::ProjectVersion]| -> Option<mcapi::hangar::ProjectVersion> {
            let mut compatible_versions = versions.iter().filter(|v| {
                if let (Some(platform), Some(platform_version)) =
                    (&current_filter.platform, &platform_version)
                {
                    v.platform_dependencies
                        .get(platform)
                        .unwrap()
                        .contains(platform_version)
                } else {
                    true
                }
            });

            if let Some(plugin_version) = plugin_version {
                compatible_versions
                    .find(|v| v.name == plugin_version)
                    .or_else(|| versions.iter().find(|v| v.name.contains(plugin_version)))
                    .cloned()
            } else {
                compatible_versions.next().cloned()
            }
        };

    loop {
        // Fetch the current page of versions.
        let versions =
            mcapi::hangar::fetch_project_versions(http_client, id, Some(current_filter.clone()))
                .await?;

        // Try to find the desired version.
        if let Some(found) = find_version(&versions.result) {
            return Ok(found);
        }

        // If we got less than `limit` items, no more pages are available.
        if versions.result.len() < current_filter.limit as usize {
            break;
        }

        // Prepare for the next page.
        current_filter.offset += current_filter.limit;
    }

    // Return a detailed error if no version was found.
    if let Some(plugin_version) = plugin_version {
        Err(anyhow!(
            "No compatible versions ('{}') for Hangar project '{}'",
            plugin_version,
            id
        ))
    } else {
        Err(anyhow!(
            "No compatible versions for Hangar project '{}'",
            id
        ))
    }
}

pub struct HangarAPI<'a>(pub &'a App);

impl HangarAPI<'_> {
    pub async fn fetch_hangar_version(&self, id: &str, version: &str) -> Result<ProjectVersion> {
        let filter = self.get_platform_filter();
        let platform_version = if filter.platform.is_some() {
            Some(self.0.mc_version().to_owned())
        } else {
            None
        };

        let version = if version == "latest" {
            get_project_version(
                &self.0.http_client,
                id,
                Some(filter),
                platform_version,
                None,
            )
            .await?
        } else if version.contains('$') {
            let version = version
                .replace("${mcver}", self.0.mc_version())
                .replace("${mcversion}", self.0.mc_version());

            get_project_version(
                &self.0.http_client,
                id,
                Some(filter),
                platform_version,
                Some(&version),
            )
            .await?
        } else {
            mcapi::hangar::fetch_project_version(&self.0.http_client, id, version).await?
        };

        Ok(version)
    }

    pub fn get_platform(&self) -> Option<mcapi::hangar::Platform> {
        match &self.0.server.jar {
            ServerType::Waterfall {} => Some(mcapi::hangar::Platform::Waterfall),
            ServerType::Velocity {} => Some(mcapi::hangar::Platform::Velocity),
            ServerType::PaperMC { project, .. } if project == "waterfall" => {
                Some(mcapi::hangar::Platform::Waterfall)
            }
            ServerType::PaperMC { project, .. } if project == "velocity" => {
                Some(mcapi::hangar::Platform::Velocity)
            }
            ServerType::PaperMC { project, .. } if project == "paper" => {
                Some(mcapi::hangar::Platform::Paper)
            }
            ServerType::Paper {} | ServerType::Purpur { .. } => {
                Some(mcapi::hangar::Platform::Paper)
            }
            _ => None,
        }
    }

    pub fn get_platform_filter(&self) -> mcapi::hangar::PlatformFilter {
        let platform = self.get_platform();
        mcapi::hangar::PlatformFilter {
            platform,
            ..Default::default()
        }
    }

    #[allow(clippy::cast_sign_loss)]
    pub async fn resolve_source(&self, id: &str, version: &str) -> Result<ResolvedFile> {
        let version = self
            .fetch_hangar_version(id, version)
            .await
            .context("Fetching project version")?;

        let download = version
            .downloads
            .get(&self.get_platform().unwrap_or(Platform::Paper))
            .ok_or(anyhow!(
                "Platform unsupported for Hangar project '{id}' version '{}'",
                version.name
            ))?;

        let cached_file_path = format!("{id}/{}/{}", version.name, download.get_file_info().name);

        Ok(ResolvedFile {
            url: download.get_url(),
            filename: download.get_file_info().name,
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed("hangar"),
                path: cached_file_path,
            },
            size: Some(download.get_file_info().size_bytes as u64),
            hashes: HashMap::from([("sha256".to_owned(), download.get_file_info().sha256_hash)]),
        })
    }
}
