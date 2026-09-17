use std::{
    collections::BTreeMap,
    io::{Read, Seek},
    path::{Component, Path, PathBuf},
};

use zip::{result::ZipError, ZipArchive};

use crate::{
    modpack::{
        mrpack::{
            error::MrpackError,
            index::{MrpackFile, MrpackIndex, FORMAT_VERSION, GAME, INDEX_NAME},
            override_path,
        },
        Side,
    },
    package::source::download::Download,
};

pub struct MrpackOverride {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

pub struct MrpackReader<R: Read + Seek> {
    pub index: MrpackIndex,
    archive: ZipArchive<R>,
}

impl<R: Read + Seek> MrpackReader<R> {
    pub fn open(mut archive: ZipArchive<R>) -> Result<Self, MrpackError> {
        let index: MrpackIndex = {
            let entry = archive.by_name(INDEX_NAME).map_err(|error| match error {
                ZipError::FileNotFound => MrpackError::NoIndex,
                error => MrpackError::Zip(error),
            })?;
            serde_json::from_reader(entry).map_err(MrpackError::ReadIndex)?
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

    pub fn overrides(
        &mut self,
        side: Option<Side>,
    ) -> Result<impl Iterator<Item = Result<MrpackOverride, MrpackError>> + '_, MrpackError> {
        let mut chosen: BTreeMap<PathBuf, (bool, usize)> = BTreeMap::new();

        for index in 0..self.archive.len() {
            let entry = self.archive.by_index(index)?;
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

            if chosen
                .get(&path)
                .is_some_and(|(chosen_side_specific, _)| *chosen_side_specific && !side_specific)
            {
                continue;
            }

            chosen.insert(path, (side_specific, index));
        }

        let archive = &mut self.archive;

        Ok(chosen.into_iter().map(move |(path, (_, index))| {
            let mut entry = archive.by_index(index)?;
            let mut bytes = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut bytes)
                .map_err(|source| MrpackError::Read {
                    name: entry.name().to_owned(),
                    source,
                })?;

            Ok(MrpackOverride { path, bytes })
        }))
    }
}

fn is_enclosed(path: &Path) -> bool {
    path.components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}
