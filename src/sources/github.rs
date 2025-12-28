use std::{
    borrow::Cow,
    collections::BTreeMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::sleep;

use crate::app::{App, CacheStrategy, ResolvedFile};

pub trait GithubRequestExt {
    fn with_token(self, token: Option<String>) -> Self;
}

impl GithubRequestExt for reqwest::RequestBuilder {
    fn with_token(self, token: Option<String>) -> Self {
        if let Some(token) = token {
            self.bearer_auth(token)
        } else {
            self
        }
    }
}

pub trait GithubWaitRatelimit<T> {
    async fn wait_ratelimit(self) -> Result<T>;
}

impl GithubWaitRatelimit<reqwest::Response> for reqwest::Response {
    async fn wait_ratelimit(self) -> Result<Self> {
        Ok(match self.headers().get("x-ratelimit-remaining") {
            Some(h) => {
                if String::from_utf8_lossy(h.as_bytes()) == "1" {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                    let ratelimit_reset =
                        String::from_utf8_lossy(self.headers()["x-ratelimit-reset"].as_bytes())
                            .parse::<u64>()?;
                    let amount = ratelimit_reset - now;
                    println!(" (!) Ratelimit exceeded. sleeping for {amount} seconds...");
                    sleep(Duration::from_secs(amount)).await;
                }

                self
            }

            None => self.error_for_status()?,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CachedData<T: Serialize> {
    pub data: T,
    pub etag: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubAsset {
    pub url: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubRepository {
    pub description: Option<String>,
}

static CACHE_DIR: &str = "github";
static GITHUB_API_VERSION: &str = "2022-11-28";

pub struct GithubAPI<'a>(pub &'a App);

impl GithubAPI<'_> {
    pub async fn fetch_api<T: DeserializeOwned + Clone + Serialize>(
        &self,
        url: String,
        cache_path: String,
    ) -> Result<T> {
        let cached_data = if let Some(cache) = self.0.get_cache(CACHE_DIR) {
            cache.try_get_json::<CachedData<T>>(&cache_path)?
        } else {
            None
        };

        let mut headers = HeaderMap::new();
        if let Some(cached_data) = &cached_data {
            headers.insert("if-none-match", HeaderValue::from_str(&cached_data.etag)?);
        }
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_str(GITHUB_API_VERSION)?,
        );

        let response = self
            .0
            .http_client
            .get(format!("{}/{url}", self.0.config.sources.github.api_url))
            .with_token(self.0.config.sources.github.api_token.clone())
            .headers(headers)
            .send()
            .await?;

        if response.status() == StatusCode::NOT_MODIFIED {
            Ok(cached_data.unwrap().data)
        } else {
            let etag = response.headers().get("etag").cloned();

            let json: T = response
                .error_for_status()?
                .wait_ratelimit()
                .await?
                .json()
                .await?;

            if let Some(etag) = etag {
                if let Some(cache) = self.0.get_cache(CACHE_DIR) {
                    cache
                        .write_json(
                            &cache_path,
                            &CachedData {
                                etag: etag.to_str()?.to_owned(),
                                data: json.clone(),
                            },
                        )
                        .context("Saving github api response to cache")?;
                }
            }

            Ok(json)
        }
    }

    pub async fn fetch_repo_description(&self, repo: &str) -> Result<String> {
        Ok(self
            .fetch_api::<GithubRepository>(
                format!("repos/{repo}"),
                format!("{repo}/repository.json"),
            )
            .await?
            .description
            .unwrap_or_default())
    }

    pub async fn fetch_releases(&self, repo: &str) -> Result<Vec<GithubRelease>> {
        self.fetch_api::<Vec<GithubRelease>>(
            format!("repos/{repo}/releases"),
            format!("{repo}/releases.json"),
        )
        .await
    }

    pub async fn fetch_release(&self, repo: &str, release_tag: &str) -> Result<GithubRelease> {
        let releases = self.fetch_releases(repo).await?;

        let tag = release_tag.replace("${mcver}", self.0.mc_version());
        let tag = tag.replace("${mcversion}", self.0.mc_version());

        let release = match tag.as_str() {
            "latest" => releases.first(),
            tag => releases
                .iter()
                .find(|r| r.tag_name == tag)
                .or_else(|| releases.iter().find(|r| r.tag_name.contains(tag))),
        }
        .ok_or(anyhow!(
            "Github release '{tag}' ('{release_tag}') not found on repository '{repo}'"
        ))?;

        Ok(release.clone())
    }

    pub async fn fetch_asset(
        &self,
        repo: &str,
        release_tag: &str,
        asset_name: &str,
    ) -> Result<(GithubRelease, GithubAsset)> {
        let release = self.fetch_release(repo, release_tag).await?;

        let asset = match asset_name {
            "" | "first" | "any" => release.assets.first(),
            id => {
                let id = if id.contains('$') {
                    id.replace("${version}", &release.tag_name)
                        .replace("${tag}", &release.tag_name)
                        .replace("${release}", &release.tag_name)
                        .replace("${mcver}", self.0.mc_version())
                        .replace("${mcversion}", self.0.mc_version())
                } else {
                    id.to_owned()
                };

                release.assets
                    .iter()
                    .find(|a| id == a.name)
                    .or(release.assets.iter().find(|a| a.name.contains(&id)))
            }
        }
        .ok_or(anyhow!(
            "Github release asset '{asset_name}' on release '{}' ('{release_tag}') of repository '{repo}' not found",
            release.tag_name
        ))?.clone();

        Ok((release, asset))
    }

    pub async fn resolve_source(
        &self,
        repo: &str,
        release_tag: &str,
        asset_name: &str,
    ) -> Result<ResolvedFile> {
        let (release, asset) = self.fetch_asset(repo, release_tag, asset_name).await?;

        let cached_file_path = format!("{repo}/releases/{}/{}", release.tag_name, asset.name);

        Ok(ResolvedFile {
            url: format!(
                "https://github.com/{repo}/releases/download/{}/{}",
                release.tag_name, asset.name
            ),
            filename: asset.name,
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed(CACHE_DIR),
                path: cached_file_path,
            },
            size: Some(asset.size),
            hashes: BTreeMap::new(),
        })
    }
}
