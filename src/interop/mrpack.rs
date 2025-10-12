use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Seek, Write},
    time::Duration,
};
use zip::{read::ZipFile, write::FileOptions, ZipArchive, ZipWriter};

use crate::{
    app::{App, Prefix, Resolvable},
    model::Downloadable,
};

pub struct MRPackInterop<'a>(pub &'a mut App);

impl MRPackInterop<'_> {
    pub async fn import_all<R: Read + Seek>(
        &mut self,
        mut mrpack: MRPackReader<R>,
        name: Option<String>,
    ) -> Result<MRPackIndex> {
        let progress_bar = self.0.multi_progress.add(ProgressBar::new_spinner());
        progress_bar.set_style(ProgressStyle::with_template("{spinner:.blue} {msg}")?);
        progress_bar.enable_steady_tick(Duration::from_millis(250));
        progress_bar.set_message("Reading index...");

        let index = mrpack.read_index()?;
        self.0.server.fill_from_map(&index.dependencies);

        progress_bar.set_message(format!("Importing {} mods...", index.files.len()));

        let pb = self
            .0
            .multi_progress
            .insert_after(&progress_bar, ProgressBar::new(index.files.len() as u64))
            .with_style(ProgressStyle::with_template(
                "  {prefix:.blue.bold} {msg} [{wide_bar:.cyan/blue}] {pos}/{len}",
            )?)
            .with_prefix("Importing");

        for file in index.files.iter().progress_with(pb.clone()) {
            pb.set_message(file.path.clone());

            let dl = if let Some(hash) = file.hashes.get("sha512") {
                if let Ok(ver) = self.0.modrinth().version_from_hash(hash, "sha512").await {
                    Some(Downloadable::Modrinth {
                        id: ver.project_id.clone(),
                        version: ver.id.clone(),
                    })
                } else {
                    None
                }
            } else {
                None
            };

            let dl = match dl {
                Some(dl) => dl,
                _ => self.0.dl_from_url(&file.downloads[0]).await?,
            };

            self.0.notify(Prefix::Imported, dl.to_short_string());
            self.0.server.mods.push(dl);
        }

        pb.finish_and_clear();

        self.0.server.save()?;

        progress_bar.set_message("Unzipping files");

        let files = mrpack.get_files();

        let pbf = self
            .0
            .multi_progress
            .insert_after(&progress_bar, ProgressBar::new(files.len() as u64))
            .with_style(ProgressStyle::with_template(
                "  {prefix:.blue.bold} {msg} [{wide_bar:.cyan/blue}] {pos}/{len}",
            )?)
            .with_prefix("Unzipping");

        for (relative_path, zip_path) in files.iter().progress_with(pbf.clone()) {
            pbf.set_message(relative_path.clone());

            let zip_file = mrpack.get_file(zip_path)?;
            let target_path = self.0.server.path.join("config").join(relative_path);

            fs::create_dir_all(target_path.parent().unwrap())?;

            // TODO mrpack import: is target_path exists prompt

            let pb = self
                .0
                .multi_progress
                .insert_after(&progress_bar, ProgressBar::new(zip_file.size()));

            let mut target_file = File::create(&target_path)?;
            io::copy(&mut pb.wrap_read(zip_file), &mut target_file)?;

            pb.finish_and_clear();
        }

        pbf.finish_and_clear();
        progress_bar.finish_and_clear();

        self.0.success(format!(
            "Imported {}",
            name.as_ref().unwrap_or(&"mrpack".to_string())
        ));

        Ok(index)
    }

    pub async fn export_all<W: Write + Seek>(&self, mut mrpack: MRPackWriter<W>) -> Result<()> {
        let progress_bar = self.0.multi_progress.add(
            ProgressBar::new_spinner().with_finish(ProgressFinish::WithMessage("Exported".into())),
        );
        progress_bar.set_message("Exporting mrpack...");
        progress_bar.enable_steady_tick(Duration::from_millis(250));

        let mut files = vec![];

        let pb = self
            .0
            .multi_progress
            .insert_after(
                &progress_bar,
                ProgressBar::new(self.0.server.mods.len() as u64).with_style(
                    ProgressStyle::with_template(
                        "{prefix:.blue.bold} {msg} [{wide_bar:.cyan/blue}] {pos}/{len}",
                    )?,
                ),
            )
            .with_prefix("Mod");
        for server_mod in self.0.server.mods.iter().progress_with(pb.clone()) {
            pb.set_message(server_mod.to_short_string());
            files.push(self.to_mrpack_file(server_mod).await?);
        }
        pb.reset();

        pb.set_prefix("Client Mod");
        for client_mod in self
            .0
            .server
            .clientsidemods
            .iter()
            .progress_with(pb.clone())
        {
            pb.set_message(if client_mod.desc.is_empty() {
                client_mod.dl.to_short_string()
            } else {
                client_mod.desc.clone()
            });
            files.push(self.to_mrpack_file(&client_mod.dl).await?);
        }
        pb.reset();

        let index = MRPackIndex {
            files,
            dependencies: self.0.server.to_map(true),
            name: self
                .0
                .var("MODPACK_NAME")
                .unwrap_or(self.0.server.name.clone()),
            summary: self.0.var("MODPACK_SUMMARY"),
            version_id: self.0.var("MODPACK_VERSION").unwrap_or_default(),
            game: "minecraft".to_owned(),
        };

        mrpack.write_index(&index)?;

        pb.set_prefix("Overrides");
        // TODO: mrpack export overrides
        pb.finish();

        mrpack.finish()?;

        self.0.success("mrpack exported!");

        Ok(())
    }

    pub async fn to_mrpack_file(&self, dl: &Downloadable) -> Result<MRPackFile> {
        let resolved = dl.resolve_source(self.0).await?;

        Ok(MRPackFile {
            path: format!("mods/{}", resolved.filename),
            hashes: resolved.hashes,
            file_size: resolved.size.unwrap_or_default(),
            // TODO: mrpack export EnvSupport
            env: None,
            downloads: vec![resolved.url],
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MRPackIndex {
    pub game: String,
    pub name: String,
    pub version_id: String,
    pub summary: Option<String>,
    pub files: Vec<MRPackFile>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MRPackFile {
    path: String,
    hashes: HashMap<String, String>,
    env: Option<Env>,
    file_size: u64,
    downloads: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Env {
    pub client: EnvSupport,
    pub server: EnvSupport,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EnvSupport {
    Required,
    Optional,
    Unsupported,
}

pub const MRPACK_INDEX_FILE: &str = "modrinth.index.json";

pub struct MRPackReader<R: Read + Seek>(pub ZipArchive<R>);

impl<R: Read + Seek> MRPackReader<R> {
    pub fn from_reader(reader: R) -> Result<Self> {
        Ok(Self(
            ZipArchive::new(reader).context("Reading mrpack zip archive")?,
        ))
    }

    pub fn read_index(&mut self) -> Result<MRPackIndex> {
        Ok(serde_json::from_reader(self.0.by_name(MRPACK_INDEX_FILE)?)?)
    }

    pub fn get_files(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        for filename in self.0.file_names().filter(|f| !f.ends_with('/')) {
            if filename.starts_with("overrides/") {
                let relative = filename.strip_prefix("overrides/").unwrap();

                if map.contains_key(relative) {
                    continue;
                }

                map.insert(relative.to_owned(), filename.to_owned());
            } else if filename.starts_with("server-overrides") {
                map.insert(
                    filename
                        .strip_prefix("server-overrides")
                        .unwrap()
                        .to_owned(),
                    filename.to_owned(),
                );
            }
        }

        map
    }

    pub fn get_file<'a>(&'a mut self, filename: &str) -> Result<ZipFile<'a>> {
        Ok(self.0.by_name(filename)?)
    }
}

pub struct MRPackWriter<W: Write + Seek>(pub ZipWriter<W>);

impl<W: Write + Seek> MRPackWriter<W> {
    pub fn from_writer(writer: W) -> Self {
        Self(ZipWriter::new(writer))
    }

    pub fn write_file(&mut self, path: &str, bytes: &[u8]) -> Result<()> {
        self.0.start_file(path, FileOptions::default())?;

        self.0.write_all(bytes)?;

        Ok(())
    }

    pub fn write_index(&mut self, index: &MRPackIndex) -> Result<()> {
        self.write_file(
            MRPACK_INDEX_FILE,
            serde_json::to_string_pretty(index)?.as_bytes(),
        )
    }

    pub fn finish(&mut self) -> Result<()> {
        self.0.finish()?;
        Ok(())
    }
}
