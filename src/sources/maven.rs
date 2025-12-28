use std::{borrow::Cow, collections::BTreeMap};

use anyhow::{anyhow, Result};

use crate::app::{App, CacheStrategy, ResolvedFile};

pub trait XMLExt {
    fn get_text(&self, k: &str) -> Result<String>;
    fn get_text_all(&self, k: &str) -> Vec<String>;
}

impl XMLExt for roxmltree::Document<'_> {
    fn get_text(&self, k: &str) -> Result<String> {
        self.descendants()
            .find_map(|elem| {
                if elem.tag_name().name() == k {
                    Some(elem.text()?.to_owned())
                } else {
                    None
                }
            })
            .ok_or(anyhow!("XML element not found: {}", k))
    }

    fn get_text_all(&self, k: &str) -> Vec<String> {
        self.descendants()
            .filter_map(|t| {
                if t.tag_name().name() == k {
                    Some(t.text()?.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MavenMetadata {
    pub latest: Option<String>,
    pub group_id: Option<String>,
    pub artifact_id: Option<String>,
    pub versions: Vec<String>,
}

impl MavenMetadata {
    pub fn find_url(&self, url: &str) -> Option<(String, String)> {
        let t = url.split_once(&format!(
            "{}/{}",
            self.group_id.clone()?.replace(['.', ':'], "/"),
            self.artifact_id.clone()?
        ))?;
        Some((t.0.to_owned(), t.1.to_owned()))
    }
}

pub struct MavenAPI<'a>(pub &'a App);

impl MavenAPI<'_> {
    pub fn get_metadata_url(url: &str, group_id: &str, artifact_id: &str) -> String {
        format!(
            "{url}/{}/{artifact_id}/maven-metadata.xml",
            group_id.replace(['.', ':'], "/")
        )
    }

    pub async fn find_maven_artifact(&self, url: &str) -> Result<MavenMetadata> {
        let metadata_url = if url.ends_with("/maven-metadata.xml") {
            url.to_string()
        } else {
            Self::guess_metadata_url(url)?
        };

        self.fetch_metadata_url(&metadata_url).await
    }

    // @author ChatGPT
    pub fn guess_metadata_url(url: &str) -> Result<String> {
        // Attempt to construct the Maven metadata URL based on the provided URL
        let segments: Vec<&str> = url.trim_end_matches('/').rsplit('/').collect();

        if let Some(last_segment) = segments.first() {
            if last_segment.is_empty() {
                // If the last segment is empty, skip it
                let metadata_url = format!("{}/maven-metadata.xml", url.trim_end_matches('/'));
                return Ok(metadata_url);
            }
        }

        if segments.len() >= 2 {
            // Construct the Maven metadata URL by going up one level
            let metadata_url = format!(
                "{}/maven-metadata.xml",
                url.trim_end_matches(segments[0]).trim_end_matches('/')
            );
            Ok(metadata_url)
        } else {
            Err(anyhow!("Invalid URL format"))
        }
    }

    pub async fn fetch_metadata(
        &self,
        url: &str,
        group_id: &str,
        artifact_id: &str,
    ) -> Result<MavenMetadata> {
        self.fetch_metadata_url(&Self::get_metadata_url(url, group_id, artifact_id))
            .await
    }

    pub async fn fetch_metadata_url(&self, url: &str) -> Result<MavenMetadata> {
        let xml = self.0.http_client.get(url).send().await?.text().await?;

        let doc = roxmltree::Document::parse(&xml)?;

        Ok(MavenMetadata {
            latest: doc.get_text("latest").ok(),
            artifact_id: doc.get_text("artifactId").ok(),
            group_id: doc.get_text("groupId").ok(),
            versions: doc.get_text_all("version"),
        })
    }

    pub async fn fetch_versions(
        &self,
        url: &str,
        group_id: &str,
        artifact_id: &str,
    ) -> Result<(String, Vec<String>)> {
        let xml = self
            .0
            .http_client
            .get(Self::get_metadata_url(url, group_id, artifact_id))
            .send()
            .await?
            .text()
            .await?;

        let doc = roxmltree::Document::parse(&xml)?;

        let latest = doc.get_text("latest").ok();

        let list = doc.get_text_all("version");

        Ok((
            latest.unwrap_or_else(|| list.first().cloned().unwrap_or_default()),
            list,
        ))
    }

    pub async fn fetch_version(
        &self,
        url: &str,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<String> {
        let (latest, versions) = self.fetch_versions(url, group_id, artifact_id).await?;

        let version = match version {
            "latest" => latest,
            id => {
                let id = id
                    .replace("${artifact}", artifact_id)
                    .replace("${mcversion}", self.0.mc_version())
                    .replace("${mcver}", self.0.mc_version());
                versions
                    .iter()
                    .find(|v| *v == &id)
                    .or_else(|| versions.iter().find(|v| v.contains(&id)))
                    .ok_or(anyhow!("Couldn't resolve maven artifact version (url={url},g={group_id},a={artifact_id})"))?
                    .clone()
            }
        };

        Ok(version)
    }

    pub async fn resolve_source(
        &self,
        url: &str,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        file: &str,
    ) -> Result<ResolvedFile> {
        let version = self
            .fetch_version(url, group_id, artifact_id, version)
            .await?;

        let file = file
            .replace("${artifact}", artifact_id)
            .replace("${version}", &version)
            .replace("${mcversion}", self.0.mc_version())
            .replace("${mcver}", self.0.mc_version());

        let file = if file.contains('.') {
            file
        } else {
            format!("{file}.jar")
        };

        let download_url = format!(
            "{url}/{}/{artifact_id}/{version}/{file}",
            group_id.replace('.', "/"),
        );

        let cached_file_path = format!(
            "{}/{}/{artifact_id}/{version}/{file}",
            crate::util::url_to_folder(url),
            group_id.replace('.', "/"),
        );

        Ok(ResolvedFile {
            url: download_url,
            filename: file,
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed("maven"),
                path: cached_file_path,
            },
            size: None,
            hashes: BTreeMap::new(),
        })
    }
}
