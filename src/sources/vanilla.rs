use crate::app::{App, CacheStrategy, ResolvedFile};
use anyhow::{anyhow, Context, Result};
use std::{borrow::Cow, collections::HashMap};

pub struct VanillaAPI<'a>(pub &'a App);

pub const CACHE_DIR: &str = "vanilla";

impl VanillaAPI<'_> {
    pub async fn fetch_latest_mcver(&self) -> Result<String> {
        Ok(mcapi::vanilla::fetch_version_manifest(&self.0.http_client)
            .await?
            .latest
            .release)
    }

    pub async fn resolve_source(&self, version: &str) -> Result<ResolvedFile> {
        let version_manifest = mcapi::vanilla::fetch_version_manifest(&self.0.http_client)
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

        let file = version
            .downloads
            .get(&mcapi::vanilla::DownloadType::Server)
            .ok_or(anyhow!(
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
