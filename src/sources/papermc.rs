use std::{borrow::Cow, collections::{BTreeMap, HashMap}};

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::app::{App, CacheStrategy, ResolvedFile};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperBuildsResponse {
    pub project_id: String,
    pub project_name: String,
    pub version: String,
    pub builds: Vec<PaperVersionBuild>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperVersionBuild {
    pub build: u64,
    pub time: String,
    pub channel: PaperChannel,
    pub promoted: bool,
    pub changes: Vec<PaperChange>,
    pub downloads: HashMap<String, PaperDownload>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperProject {
    pub project_id: String,
    pub project_name: String,
    pub version_groups: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PaperChannel {
    Default,
    Experimental,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperChange {
    pub commit: String,
    pub summary: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperDownload {
    pub name: String,
    pub sha256: String,
}

pub struct PaperMCAPI<'a>(pub &'a App);

const PAPERMC_URL: &str = "https://api.papermc.io/v2";
const CACHE_DIR: &str = "papermc";

impl PaperMCAPI<'_> {
    pub async fn fetch_api<T: DeserializeOwned + Clone + Serialize>(
        &self,
        url: String,
    ) -> Result<T> {
        let response = self.0.http_client.get(&url).send().await?;

        let json: T = response.error_for_status()?.json().await?;

        Ok(json)
    }

    pub async fn fetch_versions(&self, project: &str) -> Result<Vec<String>> {
        let proj = self
            .fetch_api::<PaperProject>(format!("{PAPERMC_URL}/projects/{project}"))
            .await?;

        Ok(proj.versions)
    }

    pub async fn fetch_builds(&self, project: &str, version: &str) -> Result<PaperBuildsResponse> {
        let resp = self
            .fetch_api(format!(
                "{PAPERMC_URL}/projects/{project}/versions/{version}/builds"
            ))
            .await?;

        Ok(resp)
    }

    pub async fn fetch_build(
        &self,
        project: &str,
        version: &str,
        build: &str,
    ) -> Result<PaperVersionBuild> {
        let builds = self.fetch_builds(project, version).await?;
        Ok(match build {
            "latest" => builds
                .builds
                .last()
                .ok_or(anyhow!(
                    "Latest papermc build for project {project} {version} not found"
                ))?
                .clone(),
            id => builds
                .builds
                .iter()
                .find(|&b| b.build.to_string() == id)
                .ok_or(anyhow!(
                    "PaperMC build '{build}' for project {project} {version} not found"
                ))?
                .clone(),
        })
    }

    pub async fn resolve_source(
        &self,
        project: &str,
        version: &str,
        build: &str,
    ) -> Result<ResolvedFile> {
        let version = match version {
            "latest" => self
                .fetch_versions(project)
                .await?
                .last()
                .ok_or(anyhow!("No versions"))?
                .clone(),
            id => id.to_owned(),
        };

        let resolved_build = self.fetch_build(project, &version, build).await?;

        let download = resolved_build.downloads.get("application")
            .ok_or(anyhow!("downloads['application'] missing for papermc project {project} {version}, build {build} ({})", resolved_build.build))?;
        let cached_file_path = format!("{project}/{}", download.name);

        Ok(ResolvedFile {
            url: format!(
                "{PAPERMC_URL}/projects/{project}/versions/{version}/builds/{}/downloads/{}",
                resolved_build.build, download.name
            ),
            filename: download.name.clone(),
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed(CACHE_DIR),
                path: cached_file_path,
            },
            size: None,
            hashes: BTreeMap::new(),
        })
    }
}
