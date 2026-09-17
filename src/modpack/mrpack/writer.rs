use std::{
    collections::BTreeMap,
    io::{Seek, Write},
    path::Path,
};

use zip::{write::FileOptions, ZipWriter};

use crate::modpack::{
    mrpack::{
        error::MrpackError,
        index::{MrpackFile, MrpackIndex, FORMAT_VERSION, GAME, INDEX_NAME},
        override_directory,
    },
    slashed, Side,
};

pub struct MrpackHeader {
    pub name: String,
    pub version_id: String,
    pub summary: Option<String>,
    pub dependencies: BTreeMap<String, String>,
}

pub struct MrpackWriter<W: Write + Seek> {
    archive: ZipWriter<W>,
    files: Vec<MrpackFile>,
}

impl<W: Write + Seek> MrpackWriter<W> {
    pub fn new(sink: W) -> Self {
        Self {
            archive: ZipWriter::new(sink),
            files: Vec::new(),
        }
    }

    pub fn add_download(&mut self, file: MrpackFile) {
        self.files.push(file);
    }

    pub fn add_override(
        &mut self,
        path: &Path,
        side: Option<Side>,
        bytes: &[u8],
    ) -> Result<(), MrpackError> {
        let name = format!("{}/{}", override_directory(side), slashed(path));

        self.write_entry(&name, bytes)
    }

    pub fn finish(mut self, header: MrpackHeader) -> Result<W, MrpackError> {
        self.files.sort_by(|left, right| left.path.cmp(&right.path));

        let index = MrpackIndex {
            format_version: FORMAT_VERSION,
            game: GAME.to_owned(),
            version_id: header.version_id,
            name: header.name,
            summary: header.summary,
            files: std::mem::take(&mut self.files),
            dependencies: header.dependencies,
        };

        let json = serde_json::to_vec_pretty(&index).map_err(MrpackError::WriteIndex)?;
        self.write_entry(INDEX_NAME, &json)?;

        Ok(self.archive.finish()?)
    }

    fn write_entry(&mut self, name: &str, bytes: &[u8]) -> Result<(), MrpackError> {
        self.archive.start_file(name, FileOptions::default())?;
        self.archive
            .write_all(bytes)
            .map_err(|source| MrpackError::Write {
                name: name.to_owned(),
                source,
            })
    }
}
