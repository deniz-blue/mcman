use std::{borrow::Cow, collections::BTreeMap, env};

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::app::{App, CacheStrategy, ResolvedFile};

static CURSEFORGE_API: &str = "https://api.curseforge.com/v1";

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeFile {
    pub id: u64,
    pub mod_id: u64,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String, // 1 = release, 2 = beta, 3 = alpha
    pub release_type: u8,
    pub file_status: u8,
    pub hashes: Vec<CurseForgeHash>,
    pub file_date: String,
    pub file_length: u64,
    pub download_count: u64,
    pub download_url: Option<String>,
    pub game_versions: Vec<String>,
    pub dependencies: Vec<CurseForgeFileDependency>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeHash {
    pub value: String,
    pub algo: u8, // 1 = sha1, 2 = md5
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeFileDependency {
    pub mod_id: u64,
    pub relation_type: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgePagination {
    pub index: u32,
    pub page_size: u32,
    pub result_count: u32,
    pub total_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurseForgeResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurseForgeListResponse<T> {
    pub data: T,
    pub pagination: CurseForgePagination,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurseForgeFilesRequest {
    #[serde(rename = "fileIds")]
    pub file_ids: Vec<u64>,
}

pub struct CurseForgeAPI<'a>(pub &'a App);

impl CurseForgeAPI<'_> {
    fn get_api_key() -> Result<String> {
        env::var("CURSEFORGE_API_KEY")
            .map_err(|_| anyhow!("CURSEFORGE_API_KEY environment variable not set."))
    }

    async fn fetch_api<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response: T = self
            .0
            .http_client
            .get(url)
            .header("x-api-key", Self::get_api_key()?)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn fetch_all_versions(&self, id: &str) -> Result<Vec<CurseForgeFile>> {
        let mut all_files = Vec::new();
        let mut index = 0u32;
        let page_size = 50u32;

        loop {
            let response: CurseForgeListResponse<Vec<CurseForgeFile>> = self
                .fetch_api(&format!(
                    "{CURSEFORGE_API}/mods/{id}/files?index={index}&pageSize={page_size}"
                ))
                .await?;

            all_files.extend(response.data);

            if all_files.len() >= response.pagination.total_count as usize {
                break;
            }

            index += page_size;
        }

        Ok(all_files)
    }

    pub fn filter_versions(&self, versions: &[CurseForgeFile]) -> Vec<CurseForgeFile> {
        let mc_version = &self.0.server.mc_version;
        let loader = self.0.server.jar.get_modrinth_name();

        versions
            .iter()
            .filter(|v| v.game_versions.iter().any(|gv| gv == mc_version))
            .filter(|v| {
                if let Some(loader_name) = loader {
                    v.game_versions.iter().any(|gv| {
                        let gv_lower = gv.to_lowercase();
                        gv_lower == loader_name
                            || (loader_name == "quilt" && gv_lower == "fabric")
                    })
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    pub async fn fetch_versions(&self, id: &str) -> Result<Vec<CurseForgeFile>> {
        let all_versions = self.fetch_all_versions(id).await?;
        Ok(self.filter_versions(&all_versions))
    }

    pub async fn fetch_files_by_ids(&self, file_ids: Vec<u64>) -> Result<Vec<CurseForgeFile>> {
        let response: CurseForgeResponse<Vec<CurseForgeFile>> = self
            .0
            .http_client
            .post(format!("{CURSEFORGE_API}/mods/files"))
            .header("x-api-key", Self::get_api_key()?)
            .json(&CurseForgeFilesRequest { file_ids })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response.data)
    }

    pub async fn fetch_version(&self, id: &str, version: &str) -> Result<CurseForgeFile> {
        if version == "latest" {
            let versions = self.fetch_versions(id).await?;
            versions.into_iter().next().ok_or_else(|| {
                anyhow!("No compatible versions for CurseForge project '{id}' (version 'latest')")
            })
        } else {
            let file_id: u64 = version
                .parse()
                .map_err(|_| anyhow!("Invalid CurseForge file ID: {version}"))?;

            let files = self.fetch_files_by_ids(vec![file_id]).await?;
            files
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("Version '{version}' not found for CurseForge project '{id}'"))
        }
    }

    fn convert_hashes(hashes: &[CurseForgeHash]) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();
        for hash in hashes {
            match hash.algo {
                1 => {
                    result.insert("sha1".to_string(), hash.value.clone());
                }
                2 => {
                    result.insert("md5".to_string(), hash.value.clone());
                }
                _ => {}
            }
        }
        result
    }

    pub async fn resolve_source(&self, project_id: &str, version: &str) -> Result<ResolvedFile> {
        let file = self.fetch_version(project_id, version).await?;

        let download_url = file.download_url.ok_or_else(|| {
            anyhow!(
                "CurseForge file {} ({}) has no download URL - may require manual download",
                file.id,
                file.display_name
            )
        })?;

        let hashes = Self::convert_hashes(&file.hashes);
        let cached_file_path = format!("{project_id}/{}/{}", file.id, file.file_name);

        Ok(ResolvedFile {
            url: download_url,
            filename: file.file_name,
            cache: CacheStrategy::File {
                namespace: Cow::Borrowed("curseforge"),
                path: cached_file_path,
            },
            size: Some(file.file_length),
            hashes,
        })
    }
}
