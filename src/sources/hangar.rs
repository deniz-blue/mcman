use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Display,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    app::{App, CacheStrategy, ResolvedFile},
    model::ServerType,
};

const API_V1: &str = "https://hangar.papermc.io/api/v1";

#[derive(Error, Debug)]
pub enum HangarError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    // #[error("{0}")]
    // APIError(String),
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Namespace {
    pub owner: String,
    pub slug: String,
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.slug)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStats {
    pub views: u64,
    pub downloads: u64,
    pub recent_views: u64,
    pub recent_downloads: u64,
    pub stars: u64,
    pub watchers: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionStats {
    pub total_downloads: u64,
    pub platform_downloads: HashMap<Platform, u64>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    AdminTools,
    Chat,
    DevTools,
    Economy,
    Gameplay,
    Games,
    Protection,
    RolePlaying,
    WorldManagement,
    Misc,
    #[default]
    Undefined,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Public,
    New,
    NeedsChanges,
    NeedsApproval,
    SoftDelete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub created_at: String,
    pub name: String,
    pub namespace: Namespace,
    pub stats: ProjectStats,
    pub category: Category,
    pub last_updated: String,
    pub visibility: Visibility,
    pub avatar_url: String,
    pub description: String,
}

pub async fn fetch_project(
    http_client: &reqwest::Client,
    id: &str,
) -> Result<Project, HangarError> {
    Ok(http_client
        .get(format!("{API_V1}/projects/{id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pagination {
    pub limit: u64,
    pub offset: u64,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectVersion {
    pub created_at: String,
    pub name: String,
    pub visibility: Visibility,
    pub description: String,
    pub stats: VersionStats,
    pub author: String,
    pub review_state: ReviewState,
    pub channel: ProjectChannel,
    pub pinned_status: PinnedStatus,
    pub downloads: HashMap<Platform, PlatformVersionDownload>,
    pub plugin_dependencies: HashMap<Platform, Vec<PluginDependency>>,
    pub platform_dependencies: HashMap<Platform, Vec<String>>,
    pub platform_dependencies_formatted: HashMap<Platform, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginDependency {
    pub name: String,
    pub required: bool,
    pub external_url: Option<String>,
    pub platform: Platform,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum Platform {
    Paper,
    Waterfall,
    Velocity,
}

impl From<&str> for Platform {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "waterfall" => Self::Waterfall,
            "velocity" => Self::Velocity,
            _ => Self::Paper,
        }
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Paper => write!(f, "PAPER"),
            Self::Waterfall => write!(f, "WATERFALL"),
            Self::Velocity => write!(f, "VELOCITY"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum PlatformVersionDownload {
    #[serde(rename_all = "camelCase")]
    Hangar {
        file_info: FileInfo,
        download_url: String,
    },

    #[serde(rename_all = "camelCase")]
    External {
        file_info: FileInfo,
        external_url: String,
    },
}

impl PlatformVersionDownload {
    #[must_use]
    pub fn get_url(&self) -> String {
        match self.clone() {
            Self::Hangar { download_url, .. } => download_url,
            Self::External { external_url, .. } => external_url,
        }
    }

    #[must_use]
    pub fn get_file_info(&self) -> FileInfo {
        match self.clone() {
            Self::Hangar { file_info, .. } | Self::External { file_info, .. } => file_info,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub name: String,
    pub size_bytes: u64,
    pub sha256_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChannel {
    pub created_at: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub flags: HashSet<ChannelFlag>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChannelFlag {
    Frozen,
    Unstable,
    Pinned,
    SendsNotifications,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum PinnedStatus {
    Version,
    Channel,
    #[default]
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    Unreviewed,
    Reviewed,
    UnderReview,
    PartiallyReviewed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlatformFilter {
    pub limit: u64,
    pub offset: u64,
    pub channel: Option<String>,
    pub platform: Option<Platform>,
}

impl Default for PlatformFilter {
    fn default() -> Self {
        Self {
            limit: 25,
            offset: 0,
            channel: None,
            platform: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectVersionsResponse {
    pub pagination: Pagination,
    pub result: Vec<ProjectVersion>,
}

pub async fn fetch_project_versions(
    http_client: &reqwest::Client,
    id: &str,
    filter: Option<PlatformFilter>,
) -> Result<ProjectVersionsResponse, HangarError> {
    let filter = filter.unwrap_or_default();

    Ok(http_client
        .get(format!(
            "{API_V1}/projects/{}/versions",
            if let Some((_, post)) = id.split_once('/') {
                post
            } else {
                id
            }
        ))
        .query(&filter)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub async fn fetch_project_version(
    http_client: &reqwest::Client,
    id: &str,
    name: &str,
) -> Result<ProjectVersion, HangarError> {
    Ok(http_client
        .get(format!(
            "{API_V1}/projects/{}/versions/{name}",
            if let Some((_, post)) = id.split_once('/') {
                post
            } else {
                id
            }
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub async fn get_project_version(
    http_client: &reqwest::Client,
    id: &str,
    filter: Option<PlatformFilter>,
    platform_version: Option<String>,
    plugin_version: Option<&str>,
) -> Result<ProjectVersion> {
    // Use the provided filter or create a default one.
    let mut current_filter = filter.unwrap_or_default();

    // Closure to search for a version in a page.
    let find_version = |versions: &[ProjectVersion]| -> Option<ProjectVersion> {
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
            fetch_project_versions(http_client, id, Some(current_filter.clone())).await?;

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
            fetch_project_version(&self.0.http_client, id, version).await?
        };

        Ok(version)
    }

    pub fn get_platform(&self) -> Option<Platform> {
        match &self.0.server.jar {
            ServerType::Waterfall {} => Some(Platform::Waterfall),
            ServerType::Velocity {} => Some(Platform::Velocity),
            ServerType::PaperMC { project, .. } if project == "waterfall" => {
                Some(Platform::Waterfall)
            }
            ServerType::PaperMC { project, .. } if project == "velocity" => {
                Some(Platform::Velocity)
            }
            ServerType::PaperMC { project, .. } if project == "paper" => Some(Platform::Paper),
            ServerType::Paper {} | ServerType::Purpur { .. } => Some(Platform::Paper),
            _ => None,
        }
    }

    pub fn get_platform_filter(&self) -> PlatformFilter {
        let platform = self.get_platform();
        PlatformFilter {
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
