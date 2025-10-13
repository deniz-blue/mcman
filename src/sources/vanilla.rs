use crate::app::{App, CacheStrategy, ResolvedFile};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

pub struct VanillaAPI<'a>(pub &'a App);

pub const CACHE_DIR: &str = "vanilla";
pub const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub id: String,
    pub assets: String,
    pub asset_index: PistonFile,
    pub java_version: VersionJavaInfo,
    pub libraries: Vec<PistonLibrary>,

    pub downloads: HashMap<DownloadType, PistonFile>,

    pub arguments: VersionArguments,
    pub minecraft_arguments: String,

    pub compliance_level: u8,
    pub minimum_launcher_version: u8,

    pub main_class: String,
    pub logging: LoggingInfoWrapper,

    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub time: String,
    pub release_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadType {
    Client,
    ClientMappings,
    Server,
    ServerMappings,
    WindowsServer,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct VersionJavaInfo {
    pub major_version: u8,
    pub component: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VersionArguments {
    pub game: Vec<PistonArgument>,
    pub jvm: Vec<PistonArgument>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum PistonArgument {
    Normal(String),
    Ruled {
        rules: Vec<PistonRule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct PistonLibrary {
    pub name: String,
    pub downloads: PistonLibraryDownload,
    pub rules: Vec<PistonRule>,

    /// Present on old versions, something like this:
    /// "extract": {
    ///     "exclude": ["META-INF/"],
    ///     "name": "tv.twitch:twitch-external-platform:4.5"
    /// }
    pub extract: Option<PistonExtractLibrary>,

    /// Present on old versions, some weird stuff involving classifiers
    /// "natives": {
    ///     "linux":   "natives-linux"
    ///     "osx":     "natives-osx"
    ///     "windows": "natives-windows-${arch}"
    /// }
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PistonExtractLibrary {
    exclude: Vec<String>,
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "action")]
pub enum PistonRule {
    Allow(PistonRuleConstraints),
    Disallow(PistonRuleConstraints),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PistonRuleConstraints {
    pub os: Option<PistonOs>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PistonOs {
    pub name: String,
    pub arch: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct PistonLibraryDownload {
    pub artifact: PistonFile,

    /// Conditional files that may be needed to be downloaded alongside the library
    /// The HashMap key specifies a classifier as additional information for downloading files
    pub classifiers: Option<HashMap<String, PistonFile>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoggingInfoWrapper {
    pub client: VersionLoggingInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VersionLoggingInfo {
    pub argument: String,
    pub file: PistonFile,
    #[serde(rename = "type")]
    pub logging_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct PistonFile {
    pub sha1: String,
    /// Size of file at url
    pub size: u64,
    pub url: String,

    /// (AssetIndex only) The game version ID the assets are for
    pub id: Option<String>,
    /// (AssetIndex only) The size of the game version's assets
    pub total_size: Option<u64>,

    /// Only present on library files
    pub path: Option<String>,
}

/// The version manifest, from piston-meta
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionIndex>,
}

impl VersionManifest {
    /// Find the version with id from the list
    #[must_use]
    pub fn find(&self, id: &str) -> Option<VersionIndex> {
        self.versions.iter().find(|v| v.id == id).cloned()
    }

    /// Fetch the latest release's `VersionInfo`
    pub async fn fetch_latest_release(&self, client: &reqwest::Client) -> Result<VersionInfo> {
        let id = self.latest.release.clone();
        self.fetch(&id, client).await
    }

    /// Fetch the latest snapshot's `VersionInfo`
    pub async fn fetch_latest_snapshot(&self, client: &reqwest::Client) -> Result<VersionInfo> {
        let id = self.latest.snapshot.clone();
        self.fetch(&id, client).await
    }

    /// Fetch the `VersionInfo` of id
    pub async fn fetch(&self, id: &str, client: &reqwest::Client) -> Result<VersionInfo> {
        self.find(id)
            .context(format!("Could not find version with ID {id}"))?
            .fetch(client)
            .await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

/// The version info from a manifest's versions list
/// Use [`Self::fetch()`] to get an [`VersionInfo`] which contains more info about the version
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionIndex {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u8,
}

impl VersionIndex {
    /// Fetch the `VersionInfo` from the manifest
    pub async fn fetch(&self, client: &reqwest::Client) -> Result<VersionInfo> {
        Ok(client
            .get(&self.url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    #[default]
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

/// Fetches the version manifest
pub async fn fetch_version_manifest(client: &reqwest::Client) -> Result<VersionManifest> {
    let version_manifest: VersionManifest = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(version_manifest)
}

impl VanillaAPI<'_> {
    pub async fn fetch_latest_mcver(&self) -> Result<String> {
        Ok(fetch_version_manifest(&self.0.http_client)
            .await?
            .latest
            .release)
    }

    pub async fn resolve_source(&self, version: &str) -> Result<ResolvedFile> {
        let version_manifest = fetch_version_manifest(&self.0.http_client)
            .await
            .context("Fetching version manifest")?;

        let version = match version {
            "latest" => version_manifest
                .fetch_latest_release(&self.0.http_client)
                .await
                .context("Fetching latest release")?,
            "latest-snapshot" => version_manifest
                .fetch_latest_snapshot(&self.0.http_client)
                .await
                .context("Fetching latest snapshot")?,
            id => version_manifest
                .fetch(id, &self.0.http_client)
                .await
                .context(format!("Fetching release {id}"))?,
        };

        let file = version.downloads.get(&DownloadType::Server).ok_or(anyhow!(
            "version manifest doesn't include a server download"
        ))?;

        let cached_file_path = format!("server-{}.jar", version.id);

        Ok(ResolvedFile {
            url: file.url.clone(),
            filename: cached_file_path.clone(),
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed(CACHE_DIR),
                path: cached_file_path,
            },
            size: Some(file.size as u64),
            hashes: HashMap::from([("sha1".to_owned(), file.sha1.clone())]),
        })
    }
}
