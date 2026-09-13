use std::{
    collections::BTreeMap,
    io::{self, Read, Seek},
    path::{Component, Path, PathBuf},
};

use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;
use zip::{result::ZipError, ZipArchive};

use crate::{
    core::checksum::Checksums,
    lockfile::Artifact,
    modpack::Side,
    package::source::download::Download,
    store::{Store, StoreError},
};

const INDEX_NAME: &str = "modrinth.index.json";
const FORMAT_VERSION: u32 = 1;
const GAME: &str = "minecraft";
const SHARED_OVERRIDES: &str = "overrides";
const CLIENT_OVERRIDES: &str = "client-overrides";
const SERVER_OVERRIDES: &str = "server-overrides";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub files: Vec<MrpackFile>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackFile {
    pub path: PathBuf,
    pub hashes: Checksums,
    #[serde(default)]
    pub env: Option<MrpackEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct MrpackEnv {
    pub client: MrpackSupport,
    pub server: MrpackSupport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MrpackSupport {
    Required,
    Optional,
    Unsupported,
    #[serde(other)]
    Unknown,
}

impl MrpackFile {
    pub fn supports(&self, side: Side) -> bool {
        let Some(env) = &self.env else {
            return true;
        };

        let support = match side {
            Side::Client => env.client,
            Side::Server => env.server,
        };

        support != MrpackSupport::Unsupported
    }

    pub fn to_download(&self) -> Download {
        Download {
            url: self.downloads.first().cloned().unwrap_or_default(),
            path: Some(self.path.clone()),
            checksums: self.hashes.clone(),
            size: Some(self.file_size),
        }
    }
}

pub struct Mrpack<R: Read + Seek> {
    pub index: MrpackIndex,
    archive: ZipArchive<R>,
}

impl<R: Read + Seek> Mrpack<R> {
    pub fn open(mut archive: ZipArchive<R>) -> Result<Self, MrpackError> {
        let index: MrpackIndex = {
            let entry = archive.by_name(INDEX_NAME).map_err(|error| match error {
                ZipError::FileNotFound => MrpackError::NoIndex,
                error => MrpackError::Zip(error),
            })?;
            serde_json::from_reader(entry).map_err(MrpackError::Index)?
        };

        if index.format_version != FORMAT_VERSION {
            return Err(MrpackError::UnsupportedFormat {
                version: index.format_version,
            });
        }

        if index.game != GAME {
            return Err(MrpackError::NotMinecraft { game: index.game });
        }

        for file in &index.files {
            if !is_enclosed(&file.path) {
                return Err(MrpackError::EscapingPath {
                    path: file.path.clone(),
                });
            }

            if file.downloads.is_empty() {
                return Err(MrpackError::NoDownloads {
                    path: file.path.clone(),
                });
            }
        }

        Ok(Self { index, archive })
    }

    pub fn downloads(&self, side: Option<Side>) -> impl Iterator<Item = Download> + '_ {
        self.index
            .files
            .iter()
            .filter(move |file| side.is_none_or(|side| file.supports(side)))
            .map(MrpackFile::to_download)
    }

    pub async fn store_overrides(
        &mut self,
        side: Option<Side>,
        store: &Store,
    ) -> Result<Vec<Artifact>, MrpackError> {
        let mut artifacts: BTreeMap<PathBuf, (bool, Artifact)> = BTreeMap::new();

        for index in 0..self.archive.len() {
            let (path, side_specific, bytes) = {
                let mut entry = self.archive.by_index(index)?;
                if entry.is_dir() {
                    continue;
                }

                let Some(name) = entry.enclosed_name() else {
                    return Err(MrpackError::EscapingPath {
                        path: PathBuf::from(entry.name()),
                    });
                };

                let Some((path, side_specific)) = override_path(name, side) else {
                    continue;
                };

                let mut bytes = Vec::with_capacity(entry.size() as usize);
                entry
                    .read_to_end(&mut bytes)
                    .map_err(|source| MrpackError::Read {
                        name: entry.name().to_owned(),
                        source,
                    })?;

                (path, side_specific, bytes)
            };

            if artifacts
                .get(&path)
                .is_some_and(|(stored_side_specific, _)| *stored_side_specific && !side_specific)
            {
                continue;
            }

            let mut object = store.write_object().await?;
            object.write(&bytes).await?;
            let hash = object.finish().await?.to_hex();

            let artifact = Artifact {
                path: path.clone(),
                hash,
                size: bytes.len() as u64,
            };
            artifacts.insert(path, (side_specific, artifact));
        }

        Ok(artifacts
            .into_values()
            .map(|(_, artifact)| artifact)
            .collect())
    }
}

fn override_path(name: &Path, side: Option<Side>) -> Option<(PathBuf, bool)> {
    if let Ok(path) = name.strip_prefix(SHARED_OVERRIDES) {
        return Some((path.to_owned(), false));
    }

    let directory = match side? {
        Side::Client => CLIENT_OVERRIDES,
        Side::Server => SERVER_OVERRIDES,
    };

    name.strip_prefix(directory)
        .ok()
        .map(|path| (path.to_owned(), true))
}

fn is_enclosed(path: &Path) -> bool {
    path.components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

#[derive(Debug, Error, Diagnostic)]
pub enum MrpackError {
    #[error("the pack has no `{INDEX_NAME}`")]
    #[diagnostic(code(mcman::mrpack_no_index))]
    NoIndex,

    #[error("the pack index could not be read")]
    #[diagnostic(code(mcman::mrpack_index))]
    Index(#[source] serde_json::Error),

    #[error("the pack uses format version {version}, only {FORMAT_VERSION} is supported")]
    #[diagnostic(code(mcman::mrpack_format))]
    UnsupportedFormat { version: u32 },

    #[error("the pack is for `{game}`, not {GAME}")]
    #[diagnostic(code(mcman::mrpack_game))]
    NotMinecraft { game: String },

    #[error("`{}` escapes the instance directory", .path.display())]
    #[diagnostic(code(mcman::mrpack_escaping_path))]
    EscapingPath { path: PathBuf },

    #[error("`{}` has no download url", .path.display())]
    #[diagnostic(code(mcman::mrpack_no_downloads))]
    NoDownloads { path: PathBuf },

    #[error("could not read `{name}` from the pack")]
    #[diagnostic(code(mcman::mrpack_read))]
    Read {
        name: String,
        #[source]
        source: io::Error,
    },

    #[error(transparent)]
    #[diagnostic(code(mcman::mrpack_zip))]
    Zip(#[from] ZipError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Store(#[from] StoreError),
}
