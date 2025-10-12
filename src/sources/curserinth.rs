use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::borrow::Cow;

use crate::app::{App, CacheStrategy, ResolvedFile};

use super::modrinth::{ModrinthFile, ModrinthProject, VersionType};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CurseRinthDependency {
    pub project_id: String,
    pub dependency_type: CurseRinthDependencyType,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum CurseRinthDependencyType {
    EmbeddedLibrary,
    OptionalDependency,
    RequiredDependency,
    Tool,
    Incompatible,
    Include,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurseRinthVersion {
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: String,
    pub dependencies: Vec<CurseRinthDependency>,
    pub game_versions: Vec<String>,
    pub version_type: VersionType,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub date_published: String,
    pub downloads: i64,
    pub files: Vec<ModrinthFile>,
}

pub static CURSERINTH_API: &str = "https://curserinth-api.kuylar.dev/v2";

pub struct CurserinthAPI<'a>(pub &'a App);

impl CurserinthAPI<'_> {
    pub async fn fetch_api<T: DeserializeOwned>(&self, url: String) -> Result<T> {
        let json: T = self
            .0
            .http_client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(json)
    }

    pub async fn fetch_project(&self, id: &str) -> Result<ModrinthProject> {
        self.fetch_api(format!("{CURSERINTH_API}/project/{id}"))
            .await
    }

    pub async fn fetch_all_versions(&self, id: &str) -> Result<Vec<CurseRinthVersion>> {
        self.fetch_api(format!("{CURSERINTH_API}/project/{id}/version"))
            .await
    }

    pub fn get_modrinth_name(&self) -> Option<&str> {
        self.0.server.jar.get_modrinth_name()
    }

    /// Result<(filtered, unfiltered)>
    pub async fn fetch_versions(
        &self,
        id: &str,
    ) -> Result<(Vec<CurseRinthVersion>, Vec<CurseRinthVersion>)> {
        let versions = self.fetch_all_versions(id).await?;

        Ok((
            versions
                .iter()
                .filter(|v| {
                    if let Some(loader) = self.get_modrinth_name() {
                        v.loaders.iter().any(|l| l.as_str() == loader)
                    } else {
                        true
                    }
                })
                .filter(|v| v.game_versions.contains(&self.0.server.mc_version))
                .cloned()
                .collect(),
            versions,
        ))
    }

    pub async fn fetch_version(&self, id: &str, version: &str) -> Result<CurseRinthVersion> {
        let (versions, unfiltered_versions) = self.fetch_versions(id).await?;

        let version = match version {
            "latest" => {
                // TODO: unfiltered_versions based on some option
                versions.first().ok_or(anyhow!(
                    "No compatible versions for CurseRinth project '{id}' (version 'latest')"
                ))?
            }
            ver => unfiltered_versions
                .iter()
                .find(|v| v.id == ver)
                .ok_or(anyhow!(
                    "Version '{ver}' not found for CurseRinth project '{id}'"
                ))?,
        };

        Ok(version.clone())
    }

    pub async fn fetch_file(
        &self,
        id: &str,
        version: &str,
    ) -> Result<(ModrinthFile, CurseRinthVersion)> {
        let version = self.fetch_version(id, version).await?;

        Ok((
            version
                .files
                .iter()
                .find(|f| f.primary)
                .ok_or(anyhow!(
                    "Primary file not found on CurseRinth version '{}' ({}), project '{id}'",
                    version.id,
                    version.name
                ))?
                .clone(),
            version,
        ))
    }

    pub async fn resolve_source(&self, id: &str, version: &str) -> Result<ResolvedFile> {
        let (file, version) = self.fetch_file(id, version).await?;

        let cached_file_path = format!("{id}/{}/{}", version.id, file.filename);

        Ok(ResolvedFile {
            url: file.url,
            filename: file.filename,
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed("curserinth"),
                path: cached_file_path,
            },
            size: Some(file.size),
            hashes: file.hashes,
        })
    }
}
