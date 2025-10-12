use std::{borrow::Cow, collections::HashMap};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::app::{App, CacheStrategy, ResolvedFile};

static SUCCESS_STR: &str = "SUCCESS";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JenkinsBuildItem {
    pub url: String,
    pub number: i32,
    pub result: String,
    #[serde(default)]
    pub fingerprint: Vec<JenkinsFingerprint>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JenkinsArtifact {
    pub file_name: String,
    pub relative_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JenkinsFingerprint {
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    hash: String,
}

pub struct JenkinsAPI<'a>(pub &'a App);

impl JenkinsAPI<'_> {
    pub fn get_url(url: &str, job: &str) -> String {
        job.split('/')
            .fold(url.strip_suffix('/').unwrap_or(url).to_owned(), |acc, j| {
                format!("{acc}/job/{j}")
            })
    }

    pub async fn fetch_builds(&self, url: &str, job: &str) -> Result<Vec<JenkinsBuildItem>> {
        Ok(serde_json::from_value(
            self.0
                .http_client
                .get(format!(
                    "{}/api/json?tree=builds[*[url,number,result]]",
                    Self::get_url(url, job)
                ))
                .send()
                .await?
                .error_for_status()?
                .json::<serde_json::Value>()
                .await?["builds"]
                .take(),
        )?)
    }

    pub async fn fetch_build(&self, url: &str, job: &str, build: &str) -> Result<JenkinsBuildItem> {
        let builds = self
            .fetch_builds(url, job)
            .await
            .context("Fetching jenkins builds")?;
        let builds = builds
            .into_iter()
            .filter(|b| b.result == SUCCESS_STR)
            .collect::<Vec<_>>();

        let selected_build = match build {
            "latest" => builds.first(),
            id => builds.iter().find(|b| b.number.to_string() == id),
        }
        .ok_or(anyhow!(
            "Can't find Jenkins build '{build}', URL: '{url}', Job: '{job}'"
        ))?
        .clone();

        Ok(selected_build)
    }

    pub async fn fetch_artifacts(&self, build_url: &str) -> Result<Vec<JenkinsArtifact>> {
        Ok(serde_json::from_value(
            self.0
                .http_client
                .get(format!("{build_url}/api/json?tree=artifacts[*]"))
                .send()
                .await?
                .error_for_status()?
                .json::<serde_json::Value>()
                .await?["artifacts"]
                .take(),
        )?)
    }

    pub async fn fetch_artifact(
        &self,
        url: &str,
        job: &str,
        build: &str,
        artifact: &str,
    ) -> Result<(JenkinsBuildItem, JenkinsArtifact)> {
        let job = &job
            .replace("${mcver}", self.0.mc_version())
            .replace("${mcversion}", self.0.mc_version());

        let selected_build = self
            .fetch_build(url, job, build)
            .await
            .context("Fetching jenkins build")?;

        let artifacts = self
            .fetch_artifacts(&selected_build.url)
            .await
            .context("Fetching jenkins artifacts")?;

        let artifact = artifact
            .replace("${mcver}", self.0.mc_version())
            .replace("${mcversion}", self.0.mc_version())
            .replace("${build}", &selected_build.number.to_string());

        let selected_artifact = match artifact.as_str() {
            "first" => artifacts.first(),
            id => artifacts.iter().find(|a| a.file_name == id)
                .or_else(|| artifacts.iter().find(|a| a.file_name.contains(id))),
        }.ok_or(anyhow!(
            "Can't find Jenkins artifact '{artifact}', on build '{}' ({build}), job '{job}', url: '{url}'",
            selected_build.number
        ))?.clone();

        Ok((selected_build, selected_artifact))
    }

    pub async fn resolve_source(
        &self,
        url: &str,
        job: &str,
        build: &str,
        artifact: &str,
    ) -> Result<ResolvedFile> {
        let (build, artifact) = self
            .fetch_artifact(url, job, build, artifact)
            .await
            .context("Fetching jenkins artifact")?;

        let cached_file_path = format!(
            "{}/{job}/{}/{}",
            crate::util::url_to_folder(url),
            build.number,
            artifact.file_name,
        );

        Ok(ResolvedFile {
            url: format!("{}artifact/{}", build.url, artifact.relative_path,),
            filename: artifact.file_name.clone(),
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed("jenkins"),
                path: cached_file_path,
            },
            size: None,
            hashes: if let Some(JenkinsFingerprint { hash, .. }) = build
                .fingerprint
                .iter()
                .find(|f| f.file_name == artifact.file_name)
            {
                HashMap::from([("md5".to_owned(), hash.clone())])
            } else {
                HashMap::new()
            },
        })
    }

    pub async fn fetch_description(&self, url: &str, job: &str) -> Result<String> {
        Ok(self
            .0
            .http_client
            .get(format!(
                "{}/api/json?tree=description",
                Self::get_url(url, job)
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?["description"]
            .take()
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_default())
    }
}
