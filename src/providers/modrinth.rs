use std::fmt::Display;

use miette::{Context, IntoDiagnostic, Result};
use reqwest_middleware::ClientWithMiddleware;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    addons::{modrinth::ModrinthAddon, platform::minecraft::MinecraftPlatform, Platform},
    core::checksum::{ChecksumAlgorithm, Checksums},
    package::{source::download::Download, Package},
    providers::{AddonResolver, ProviderError, Resolved},
};

pub const MODRINTH_API: &str = "https://api.modrinth.com";

/// Modrinth accepts a slug wherever it accepts an id, so `modrinth:luckperms` needs no lookup.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModrinthProjectId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModrinthVersionId(pub String);

impl ModrinthProjectId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ModrinthProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Display for ModrinthVersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModrinthProject {
    pub id: ModrinthProjectId,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub client_side: ModrinthSideSupport,
    pub server_side: ModrinthSideSupport,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub versions: Vec<ModrinthVersionId>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModrinthVersion {
    pub id: ModrinthVersionId,
    pub project_id: ModrinthProjectId,
    pub name: String,
    pub version_number: String,
    pub version_type: ModrinthVersionType,
    pub date_published: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<ModrinthFile>,
    pub dependencies: Vec<ModrinthDependency>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    pub hashes: ModrinthFileHashes,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModrinthFileHashes {
    pub sha1: String,
    pub sha512: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModrinthDependency {
    pub project_id: Option<ModrinthProjectId>,
    pub version_id: Option<ModrinthVersionId>,
    pub file_name: Option<String>,
    pub dependency_type: ModrinthDependencyType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModrinthVersionType {
    Release,
    Beta,
    Alpha,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModrinthDependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModrinthSideSupport {
    Required,
    Optional,
    Unsupported,
    #[serde(other)]
    Unknown,
}

impl ModrinthVersionType {
    fn rank(self) -> u8 {
        match self {
            Self::Release => 2,
            Self::Beta => 1,
            Self::Alpha | Self::Unknown => 0,
        }
    }

    pub fn is_at_least(self, floor: Self) -> bool {
        self.rank() >= floor.rank()
    }
}

impl ModrinthVersion {
    pub fn primary_file(&self) -> Option<&ModrinthFile> {
        self.files
            .iter()
            .find(|file| file.primary)
            .or(self.files.first())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ModrinthVersionQuery {
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub featured: Option<bool>,
}

impl ModrinthVersionQuery {
    pub fn parameters(&self) -> Vec<(&'static str, String)> {
        let mut parameters = Vec::new();

        if !self.loaders.is_empty() {
            parameters.push(("loaders", json_array(&self.loaders)));
        }

        if !self.game_versions.is_empty() {
            parameters.push(("game_versions", json_array(&self.game_versions)));
        }

        if let Some(featured) = self.featured {
            parameters.push(("featured", featured.to_string()));
        }

        parameters
    }
}

fn json_array(values: &[String]) -> String {
    serde_json::to_string(values).expect("a list of strings is always valid JSON")
}

pub struct Modrinth {
    http: ClientWithMiddleware,
    api: String,
}

impl Modrinth {
    pub fn new(http: ClientWithMiddleware) -> Self {
        Self {
            http,
            api: MODRINTH_API.to_owned(),
        }
    }

    pub fn with_api(http: ClientWithMiddleware, api: impl Into<String>) -> Self {
        Self {
            http,
            api: api.into(),
        }
    }

    pub async fn project(&self, project: &ModrinthProjectId) -> Result<ModrinthProject> {
        self.get(&format!("v2/project/{project}"), &[]).await
    }

    /// Modrinth returns these newest first.
    pub async fn versions(
        &self,
        project: &ModrinthProjectId,
        query: &ModrinthVersionQuery,
    ) -> Result<Vec<ModrinthVersion>> {
        self.get(
            &format!("v2/project/{project}/version"),
            &query.parameters(),
        )
        .await
    }

    pub async fn version(&self, version: &ModrinthVersionId) -> Result<ModrinthVersion> {
        self.get(&format!("v2/version/{version}"), &[]).await
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        parameters: &[(&str, String)],
    ) -> Result<T> {
        let url = format!("{}/{path}", self.api);

        self.http
            .get(&url)
            .query(parameters)
            .send()
            .await
            .into_diagnostic()
            .and_then(|response| response.error_for_status().into_diagnostic())
            .wrap_err_with(|| format!("requesting {url}"))?
            .json()
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("reading {url}"))
    }
}

impl AddonResolver for Modrinth {
    type Addon = ModrinthAddon;

    async fn resolve(
        &self,
        addon: &ModrinthAddon,
        platforms: &[Platform],
    ) -> Result<Resolved<ModrinthAddon>, ProviderError> {
        if platforms.is_empty() {
            return Err(ProviderError::PlatformRequired);
        }

        let query = ModrinthVersionQuery {
            loaders: loaders_for(platforms),
            game_versions: MinecraftPlatform::declared_in(platforms)
                .and_then(|minecraft| minecraft.version.clone())
                .into_iter()
                .collect(),
            featured: None,
        };

        let versions = self
            .versions(&ModrinthProjectId(addon.id.clone()), &query)
            .await
            .map_err(|report| ProviderError::Request(report.into()))?;

        let requested = addon.version.as_deref();
        let version =
            select_version(&versions, requested).ok_or(ProviderError::NoMatchingVersion {
                requested: requested.unwrap_or("latest").to_owned(),
            })?;
        let files = select_files(version, &addon.files)?;

        Ok(to_resolved(addon, version, &files))
    }
}

pub fn loaders_for(platforms: &[Platform]) -> Vec<String> {
    let mut loaders: Vec<String> = Vec::new();

    for platform in platforms {
        for name in platform.accepts() {
            if !loaders.iter().any(|loader| loader == name) {
                loaders.push((*name).to_owned());
            }
        }
    }

    loaders
}

pub fn select_version<'a>(
    versions: &'a [ModrinthVersion],
    requested: Option<&str>,
) -> Option<&'a ModrinthVersion> {
    let floor = match requested {
        None | Some("latest") => ModrinthVersionType::Release,
        Some("beta") => ModrinthVersionType::Beta,
        Some("alpha") => ModrinthVersionType::Alpha,
        Some(exact) => {
            return versions
                .iter()
                .find(|version| version.version_number == exact)
        }
    };

    versions
        .iter()
        .find(|version| version.version_type.is_at_least(floor))
}

pub fn select_files<'a>(
    version: &'a ModrinthVersion,
    wanted: &[String],
) -> Result<Vec<&'a ModrinthFile>, ProviderError> {
    if wanted.is_empty() {
        return Ok(version.primary_file().into_iter().collect());
    }

    wanted
        .iter()
        .map(|name| {
            version
                .files
                .iter()
                .find(|file| &file.filename == name)
                .ok_or_else(|| ProviderError::NoSuchFile { file: name.clone() })
        })
        .collect()
}

pub fn to_resolved(
    addon: &ModrinthAddon,
    version: &ModrinthVersion,
    files: &[&ModrinthFile],
) -> Resolved<ModrinthAddon> {
    let resolved = ModrinthAddon {
        id: addon.id.clone(),
        version: Some(version.version_number.clone()),
        files: files.iter().map(|file| file.filename.clone()).collect(),
    };

    let downloads = files
        .iter()
        .map(|file| {
            let mut checksums = Checksums::default();
            checksums.insert(ChecksumAlgorithm::Sha1, file.hashes.sha1.clone());
            checksums.insert(ChecksumAlgorithm::Sha512, file.hashes.sha512.clone());

            Download {
                url: file.url.clone(),
                path: Some(file.filename.clone().into()),
                checksums,
                size: Some(file.size),
            }
        })
        .collect();

    Resolved::new(resolved, Package::from_downloads(downloads))
}
