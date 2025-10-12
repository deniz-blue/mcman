use std::{borrow::Cow, collections::HashMap, time::Duration};

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    app::{App, CacheStrategy, ResolvedFile},
    model::{ServerType, SoftwareType},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModrinthProject {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: DependencyType,
    pub server_side: DependencyType,
    //pub body: String,
    pub project_type: String,
    // ...
    #[serde(default = "empty")]
    pub id: String,
    //pub team: String,
    pub versions: Vec<String>,
}

fn empty() -> String {
    String::new()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModrinthSearchResults {
    pub hits: Vec<ModrinthProject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModrinthVersion {
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub dependencies: Vec<ModrinthDependency>,
    pub game_versions: Vec<String>,
    pub version_type: VersionType,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub status: ModrinthStatus,
    pub requested_status: Option<ModrinthStatus>,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub date_published: String,
    pub downloads: u64,
    pub files: Vec<ModrinthFile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: Option<DependencyType>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
    Unsupported,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ModrinthStatus {
    Listed,
    Archived,
    Draft,
    Unlisted,
    Scheduled,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModrinthFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    // file_type omitted
}

pub trait ModrinthWaitRatelimit<T> {
    async fn wait_ratelimit(self) -> Result<T>;
}

impl ModrinthWaitRatelimit<reqwest::Response> for reqwest::Response {
    async fn wait_ratelimit(self) -> Result<Self> {
        let res = if let Some(h) = self.headers().get("x-ratelimit-remaining") {
            if String::from_utf8_lossy(h.as_bytes()) == "1" {
                let ratelimit_reset =
                    String::from_utf8_lossy(self.headers()["x-ratelimit-reset"].as_bytes())
                        .parse::<u64>()?;
                let amount = ratelimit_reset;
                println!(" (!) Ratelimit exceeded. sleeping for {amount} seconds...");
                sleep(Duration::from_secs(amount)).await;
            }
            self
        } else {
            self.error_for_status()?
        };

        Ok(res)
    }
}

pub struct ModrinthAPI<'a>(pub &'a App);

static API_URL: &str = "https://api.modrinth.com/v2";

impl ModrinthAPI<'_> {
    pub async fn fetch_api<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let json: T = self
            .0
            .http_client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .wait_ratelimit()
            .await?
            .json()
            .await?;

        Ok(json)
    }

    pub async fn fetch_project(&self, id: &str) -> Result<ModrinthProject> {
        self.fetch_api(&format!("{API_URL}/project/{id}")).await
    }

    pub async fn fetch_all_versions(&self, id: &str) -> Result<Vec<ModrinthVersion>> {
        self.fetch_api(&format!("{API_URL}/project/{id}/version"))
            .await
    }

    pub async fn fetch_versions(&self, id: &str) -> Result<Vec<ModrinthVersion>> {
        let versions = self.fetch_all_versions(id).await?;

        Ok(self.filter_versions(&versions))
    }

    pub async fn fetch_version(&self, id: &str, version: &str) -> Result<ModrinthVersion> {
        let all_versions = self.fetch_all_versions(id).await?;
        let versions = self.filter_versions(&all_versions);

        let ver = version.replace("${mcver}", self.0.mc_version());
        let ver = ver.replace("${mcversion}", self.0.mc_version());

        let version_data = if let Some(v) = match ver.as_str() {
            "latest" => versions.first(),
            ver => versions
                .iter()
                .find(|v| v.id == ver || v.name == ver || v.version_number == ver),
        } {
            v.clone()
        } else {
            let v = match ver.as_str() {
                "latest" => all_versions.first(),
                ver => all_versions
                    .iter()
                    .find(|v| v.id == ver || v.name == ver || v.version_number == ver),
            }
            .ok_or(anyhow!(
                "Couln't find version '{ver}' ('{version}') for Modrinth project '{id}'"
            ))?
            .clone();
            self.0.warn(format!(
                "Filtering failed for modrinth.com/mod/{id}/version/{ver}"
            ));
            v
        };

        Ok(version_data)
    }

    pub async fn fetch_file(
        &self,
        id: &str,
        version: &str,
    ) -> Result<(ModrinthFile, ModrinthVersion)> {
        let version = self.fetch_version(id, version).await?;

        Ok((
            version
                .files
                .iter()
                .find(|f| f.primary)
                .or(version.files.first())
                .ok_or(anyhow!(
                    "No file found on modrinth:{id}/{} ({})",
                    version.id,
                    version.name
                ))?
                .clone(),
            version,
        ))
    }

    pub fn get_modrinth_name(&self) -> Option<&str> {
        self.0.server.jar.get_modrinth_name()
    }

    pub fn get_modrinth_facets(&self) -> String {
        let mut arr: Vec<Vec<String>> = vec![];

        if self.0.server.jar.get_software_type() != SoftwareType::Proxy {
            arr.push(vec![format!("versions:{}", self.0.mc_version())]);
        }

        if let Some(n) = self.get_modrinth_name() {
            arr.push(vec![format!("categories:{n}")]);
            if n == "quilt" {
                arr.push(vec![format!("categories:fabric")]);
            }
        }

        serde_json::to_string(&arr).unwrap()
    }

    pub fn filter_versions(&self, list: &[ModrinthVersion]) -> Vec<ModrinthVersion> {
        let is_proxy = self.0.server.jar.get_software_type() == SoftwareType::Proxy;
        let is_vanilla = matches!(self.0.server.jar, ServerType::Vanilla {});

        let mcver = self.0.mc_version();
        let loader = self.get_modrinth_name();

        list.iter()
            .filter(|v| is_proxy || v.game_versions.iter().any(|s| s.as_str() == mcver))
            .filter(|v| {
                if let Some(n) = loader {
                    v.loaders
                        .iter()
                        .any(|l| l == "datapack" || l == n || (l == "fabric" && n == "quilt"))
                } else if is_vanilla {
                    v.loaders.iter().any(|s| s.as_str() == "datapack")
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    pub async fn search(&self, query: &str) -> Result<Vec<ModrinthProject>> {
        Ok(self
            .0
            .http_client
            .get(format!("{API_URL}/search"))
            .query(&[("query", query), ("facets", &self.get_modrinth_facets())])
            .send()
            .await?
            .error_for_status()?
            .json::<ModrinthSearchResults>()
            .await?
            .hits)
    }

    pub async fn version_from_hash(&self, hash: &str, algo: &str) -> Result<ModrinthVersion> {
        self.fetch_api(&format!(
            "{API_URL}/version_file/{hash}{}",
            if algo.is_empty() || algo == "sha1" {
                String::new()
            } else {
                format!("?algorithm={algo}")
            }
        ))
        .await
    }

    pub async fn resolve_source(&self, id: &str, version: &str) -> Result<ResolvedFile> {
        let (file, version) = self.fetch_file(id, version).await?;

        let cached_file_path = format!("{id}/{}/{}", version.id, file.filename);

        Ok(ResolvedFile {
            url: file.url,
            filename: file.filename,
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed("modrinth"),
                path: cached_file_path,
            },
            size: Some(file.size),
            hashes: file.hashes,
        })
    }
}
