use std::path::PathBuf;

use crate::{
    core::{checksum::Checksums, location::Location, AppContext},
    modpack::{
        is_enclosed,
        packwiz::{
            error::PackwizError,
            metafile::PackwizMetafile,
            pack::{PackwizFile, PackwizIndex, PackwizPack},
        },
    },
};

pub struct PackwizPackReader {
    location: Location,
    bytes: Vec<u8>,
    pub pack: PackwizPack,
}

pub struct PackwizIndexReader {
    location: Location,
    pub pack: PackwizPack,
    index: PackwizIndex,
}

pub struct PackwizEntry<'a> {
    reader: &'a PackwizIndexReader,
    file: &'a PackwizFile,
}

pub enum PackwizContent {
    Metafile(PackwizMetafile),
    File(Vec<u8>),
}

impl PackwizPackReader {
    pub async fn open(ctx: &AppContext, location: Location) -> Result<Self, PackwizError> {
        let bytes = location.read_bytes(ctx).await?;
        let pack = location.parse_toml(&bytes)?;

        Ok(Self {
            location,
            bytes,
            pack,
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub async fn fetch_index(self, ctx: &AppContext) -> Result<PackwizIndexReader, PackwizError> {
        let location = self.location.join(&self.pack.index.file)?;
        let index: PackwizIndex = location.read_toml(ctx).await?;

        for file in &index.files {
            let path = PathBuf::from(&file.file);
            if !is_enclosed(&path) {
                return Err(PackwizError::EscapingPath { path });
            }
        }

        Ok(PackwizIndexReader {
            location,
            pack: self.pack,
            index,
        })
    }
}

impl PackwizIndexReader {
    pub fn entries(&self) -> impl Iterator<Item = PackwizEntry<'_>> {
        self.index
            .files
            .iter()
            .map(|file| PackwizEntry { reader: self, file })
    }
}

impl PackwizEntry<'_> {
    pub fn path(&self) -> PathBuf {
        PathBuf::from(&self.file.file)
    }

    pub async fn read(&self, ctx: &AppContext) -> Result<PackwizContent, PackwizError> {
        let location = self.reader.location.join(&self.file.file)?;
        let bytes = location.read_bytes(ctx).await?;

        let mut declared = Checksums::default();
        declared.insert(self.reader.index.hash_format, self.file.hash.clone());
        declared.verify(&bytes)?;

        if self.file.metafile {
            return Ok(PackwizContent::Metafile(location.parse_toml(&bytes)?));
        }

        Ok(PackwizContent::File(bytes))
    }
}
