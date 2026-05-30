use std::{borrow::Cow, collections::{BTreeMap, HashMap}};

use anyhow::{anyhow, bail, Result};
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::app::{App, CacheStrategy, ResolvedFile};

// PaperMC migrated server distribution off the legacy `api.papermc.io/v2`
// service onto the new "Fill" API at `fill.papermc.io/v3`. v2 stopped
// receiving new versions and returns HTTP 404 for Minecraft releases above
// 1.21.11 (e.g. 26.1.2), whereas v3 has the builds. The v3 shapes differ from
// v2: builds are returned as a bare newest-first array, build ids live under
// `id`, the server jar is keyed `server:default` in `downloads`, each download
// carries `name`/`size`/`url`/`checksums.sha256` (so the URL no longer has to be
// constructed by hand), and a project's `versions` is an ordered map of
// version-group -> versions. This module is rewritten to target that v3 shape.

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperBuild {
    pub id: u64,
    pub time: String,
    // Retained (e.g. "STABLE") for forward-compat; not currently used for filtering.
    pub channel: String,
    pub downloads: HashMap<String, PaperDownload>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperDownload {
    pub name: String,
    pub size: u64,
    pub url: String,
    pub checksums: PaperChecksums,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperChecksums {
    pub sha256: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaperProject {
    // Ordered (newest-first) map of version-group -> list of versions, e.g.
    // {"26.1": ["26.1.2", "26.1.1"], "1.21": ["1.21.11", ...], ...}.
    // IndexMap preserves the API's insertion order so "latest" is reliable.
    pub versions: IndexMap<String, Vec<String>>,
}

pub struct PaperMCAPI<'a>(pub &'a App);

const PAPERMC_URL: &str = "https://fill.papermc.io/v3";
const CACHE_DIR: &str = "papermc";

// `project`/`version`/`build` originate from user config and are interpolated
// into `fill.papermc.io` URL paths, so reject anything outside a conservative
// allowlist to avoid path traversal / SSRF-adjacent requests.
fn validate_segment(kind: &str, s: &str) -> Result<()> {
    if s.is_empty() || s.chars().any(|c| !matches!(c, 'a'..='z'|'A'..='Z'|'0'..='9'|'.'|'-'|'_')) {
        bail!("Invalid PaperMC {kind}: {s:?}");
    }
    Ok(())
}

impl PaperMCAPI<'_> {
    pub async fn fetch_api<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.0.http_client.get(url).send().await?;

        let json: T = response.error_for_status()?.json().await?;

        Ok(json)
    }

    pub async fn fetch_versions(&self, project: &str) -> Result<PaperProject> {
        validate_segment("project", project)?;
        self.fetch_api::<PaperProject>(&format!("{PAPERMC_URL}/projects/{project}"))
            .await
    }

    pub async fn fetch_builds(&self, project: &str, version: &str) -> Result<Vec<PaperBuild>> {
        validate_segment("project", project)?;
        validate_segment("version", version)?;
        // v3 returns a bare JSON array of builds, newest-first.
        self.fetch_api(&format!(
            "{PAPERMC_URL}/projects/{project}/versions/{version}/builds"
        ))
        .await
    }

    pub async fn fetch_build(
        &self,
        project: &str,
        version: &str,
        build: &str,
    ) -> Result<PaperBuild> {
        match build {
            // Builds are newest-first, so "latest" is the first element of the
            // builds array.
            "latest" => {
                let builds = self.fetch_builds(project, version).await?;
                builds
                    .first()
                    .ok_or(anyhow!(
                        "Latest papermc build for project {project} {version} not found"
                    ))
                    .cloned()
            }
            // Pinned build: hit the single-build endpoint directly.
            id => {
                validate_segment("project", project)?;
                validate_segment("version", version)?;
                validate_segment("build", id)?;
                self.fetch_api(&format!(
                    "{PAPERMC_URL}/projects/{project}/versions/{version}/builds/{id}"
                ))
                .await
            }
        }
    }

    pub async fn resolve_source(
        &self,
        project: &str,
        version: &str,
        build: &str,
    ) -> Result<ResolvedFile> {
        let version = match version {
            "latest" => {
                let proj = self.fetch_versions(project).await?;
                // `versions` is newest-first; the first group's first entry is
                // the newest version overall.
                proj.versions
                    .values()
                    .next()
                    .and_then(|v| v.first())
                    .ok_or(anyhow!("No versions for papermc project {project}"))?
                    .clone()
            }
            id => id.to_owned(),
        };

        let resolved_build = self.fetch_build(project, &version, build).await?;

        let download = resolved_build.downloads.get("server:default").ok_or(anyhow!(
            "downloads['server:default'] missing for papermc project {project} {version}, build {build} ({})",
            resolved_build.id
        ))?;

        // Cache layout: <namespace>/<project>/<filename>, e.g. papermc/paper/paper-26.1.2-66.jar
        let cached_file_path = format!("{project}/{}", download.name);

        Ok(ResolvedFile {
            // v3 provides the download URL directly; use it verbatim.
            url: download.url.clone(),
            filename: download.name.clone(),
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed(CACHE_DIR),
                path: cached_file_path,
            },
            size: Some(download.size),
            hashes: BTreeMap::from([(
                String::from("sha256"),
                download.checksums.sha256.clone(),
            )]),
        })
    }
}
