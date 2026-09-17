use std::{collections::BTreeMap, path::Path};

use miette::{bail, IntoDiagnostic, Result, WrapErr};
use serde::Serialize;

use crate::{
    core::{
        checksum::{ChecksumAlgorithm, Checksums},
        fs::{create_parent, link_or_copy},
    },
    modpack::{
        packwiz::{
            metafile::PackwizMetafile,
            pack::{PackwizFile, PackwizIndex, PackwizPack, INDEX_TOML, PACK_FORMAT, PACK_TOML},
        },
        slashed,
    },
};

const INDEX_HASHES: ChecksumAlgorithm = ChecksumAlgorithm::Sha256;

pub struct PackwizPackHeader {
    pub name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub versions: BTreeMap<String, String>,
}

pub struct PackwizPackWriter<'a> {
    root: &'a Path,
    files: Vec<PackwizFile>,
}

impl<'a> PackwizPackWriter<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self {
            root,
            files: Vec::new(),
        }
    }

    pub async fn add_file(&mut self, path: &Path, source: &Path) -> Result<()> {
        link_or_copy(source, &self.root.join(path)).await?;

        let bytes = tokio::fs::read(source)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("could not read `{}`", source.display()))?;

        self.push(path, INDEX_HASHES.digest(&bytes), false);

        Ok(())
    }

    pub fn index_file(&mut self, path: &Path, checksums: &Checksums) -> Result<()> {
        let Some(digest) = checksums.get(INDEX_HASHES) else {
            bail!(
                "`{}` has no {} to put in the packwiz index",
                path.display(),
                INDEX_HASHES.name()
            );
        };

        self.push(path, digest.to_owned(), false);

        Ok(())
    }

    pub async fn write_metafile(
        &mut self,
        destination: &Path,
        metafile: &PackwizMetafile,
    ) -> Result<()> {
        let path = destination.with_extension("pw.toml");

        let digest = self.write_toml(&path, metafile).await?;
        self.push(&path, digest, true);

        Ok(())
    }

    pub async fn finish(mut self, header: PackwizPackHeader) -> Result<()> {
        self.files.sort_by(|left, right| left.file.cmp(&right.file));

        let index = PackwizIndex {
            hash_format: INDEX_HASHES,
            files: std::mem::take(&mut self.files),
        };

        let path = Path::new(INDEX_TOML);
        let digest = self.write_toml(path, &index).await?;

        let pack = PackwizPack {
            name: header.name,
            author: header.author,
            version: header.version,
            description: header.description,
            pack_format: PACK_FORMAT.to_owned(),
            index: PackwizFile {
                file: slashed(path),
                hash: digest,
                metafile: false,
            },
            versions: header.versions,
        };

        self.write_toml(Path::new(PACK_TOML), &pack).await?;

        Ok(())
    }

    fn push(&mut self, path: &Path, hash: String, metafile: bool) {
        self.files.push(PackwizFile {
            file: slashed(path),
            hash,
            metafile,
        });
    }

    async fn write_toml(&self, path: &Path, value: &impl Serialize) -> Result<String> {
        let text = toml::to_string_pretty(value)
            .into_diagnostic()
            .wrap_err_with(|| format!("could not write `{}`", path.display()))?;

        let destination = self.root.join(path);
        create_parent(&destination).await?;

        tokio::fs::write(&destination, &text)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("could not write `{}`", destination.display()))?;

        Ok(INDEX_HASHES.digest(text.as_bytes()))
    }
}
