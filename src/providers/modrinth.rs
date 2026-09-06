use std::fmt::Display;

use miette::{Context, IntoDiagnostic, Result};
use reqwest_middleware::ClientWithMiddleware;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const MODRINTH_API: &str = "https://api.modrinth.com";

/// Modrinth takes a slug wherever it takes an id, so `modrinth:luckperms` can ask for
/// versions without looking the project up first.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModrinthProjectId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModrinthVersionId(pub String);

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModrinthDependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModrinthSideSupport {
    Required,
    Optional,
    Unsupported,
    Unknown,
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
