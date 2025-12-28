use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use pathdiff::diff_paths;
use rpackwiz::model::{
    DownloadMode, HashFormat, Mod, ModDownload, ModUpdate, Pack, PackFile, PackIndex,
};
use serde::de::DeserializeOwned;
use tokio::{fs::File, io::AsyncWriteExt};
use walkdir::WalkDir;

use crate::{
    app::{AddonType, App, CacheStrategy, Prefix, ProgressPrefix, Resolvable, ResolvedFile},
    model::Downloadable,
    util::env::try_get_url,
};

#[derive(Debug, Clone)]
pub enum FileProvider {
    LocalFolder(PathBuf),
    RemoteURL(reqwest::Client, reqwest::Url),
}

impl FileProvider {
    pub async fn parse_toml<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        match self {
            Self::LocalFolder(folder) => {
                let str = tokio::fs::read_to_string(folder.join(path)).await?;
                Ok(toml::from_str(&str)?)
            }
            Self::RemoteURL(http_client, url) => {
                let contents = http_client
                    .get(url.join(path)?)
                    .send()
                    .await?
                    .error_for_status()?
                    .text()
                    .await?;

                Ok(toml::from_str(&contents)?)
            }
        }
    }
}

pub struct PackwizInterop<'a>(pub &'a mut App);

impl PackwizInterop<'_> {
    pub fn get_file_provider(&self, s: &str) -> Result<FileProvider> {
        Ok(if s.starts_with("http") {
            FileProvider::RemoteURL(self.0.http_client.clone(), s.try_into()?)
        } else {
            FileProvider::LocalFolder(s.into())
        })
    }

    pub async fn import_all(&mut self, from: &str) -> Result<()> {
        self.import_from_source(self.get_file_provider(from)?).await
    }

    pub async fn import_from_source(&mut self, source: FileProvider) -> Result<()> {
        let progress_bar = self.0.multi_progress.add(ProgressBar::new_spinner());
        progress_bar.set_style(ProgressStyle::with_template("{spinner:.blue} {msg}")?);
        progress_bar.set_message("Reading pack.toml...");
        progress_bar.enable_steady_tick(Duration::from_millis(250));

        let pack: Pack = source.parse_toml("pack.toml").await?;

        self.0.server.fill_from_map(&pack.versions);

        progress_bar.set_message("Reading pack index...");

        let index: PackIndex = source.parse_toml(&pack.index.file).await?;

        progress_bar.set_message(format!("Importing {} files...", index.files.len()));

        let pb = self
            .0
            .multi_progress
            .insert_after(&progress_bar, ProgressBar::new(index.files.len() as u64))
            .with_style(ProgressStyle::with_template(
                "  {prefix:.blue.bold} {msg} [{wide_bar:.cyan/blue}] {pos}/{len}",
            )?)
            .with_prefix("Importing");

        for file in index.files.iter().progress_with(pb.clone()) {
            pb.set_message(file.file.clone());
            if file.metafile {
                if file.file.starts_with("mods") {
                    let modpw: Mod = source.parse_toml(&file.file).await?;

                    let dl = self.dl_from_mod(&modpw).await?;
                    let dl_name = dl.to_short_string();

                    self.0.add_addon(AddonType::Mod, dl)?;
                    self.0.notify(Prefix::Imported, dl_name);
                } else {
                    // TODO: ???
                    self.0.warn(format!(
                        "unsupported metafile: {} - please open an issue at github",
                        file.file
                    ));
                }
            } else {
                let output_path = self.0.server.path.join("config").join(&file.file);

                match &source {
                    FileProvider::LocalFolder(folder) => {
                        tokio::fs::copy(folder.join(&file.file), output_path).await?;
                    }
                    FileProvider::RemoteURL(_, url) => {
                        self.0
                            .download_resolved(
                                ResolvedFile {
                                    url: url.join(&file.file)?.as_str().to_owned(),
                                    filename: file.file.split('/').next_back().unwrap().to_owned(),
                                    cache: CacheStrategy::None,
                                    size: None,
                                    hashes: BTreeMap::from([(
                                        match index.hash_format {
                                            HashFormat::Md5 => "md5",
                                            HashFormat::Sha1 => "sha1",
                                            HashFormat::Sha256 => "sha256",
                                            HashFormat::Sha512 => "sha512",
                                            HashFormat::Curseforge => "murmur2",
                                            _ => unreachable!(),
                                        }
                                        .to_owned(),
                                        file.hash.clone(),
                                    )]),
                                },
                                output_path.parent().unwrap().to_path_buf(),
                                self.0
                                    .multi_progress
                                    .insert_after(&pb, ProgressBar::new_spinner()),
                            )
                            .await?;
                    }
                }
            }
        }

        progress_bar.finish_and_clear();
        self.0.success("Packwiz pack imported!");

        Ok(())
    }

    pub async fn dl_from_mod(&self, m: &Mod) -> Result<Downloadable> {
        if let Some(dl) = self.dl_from_hash(&m.download).await? {
            Ok(dl)
        } else if let Some(dl) = self.dl_from_mod_update(m.update.as_ref()) {
            Ok(dl)
        } else {
            self.0
                .dl_from_string(
                    &m.download
                        .url
                        .clone()
                        .ok_or(anyhow!("Download URL not present for mod: {m:#?}"))?,
                )
                .await
                .context(format!("Importing mod: {m:#?}"))
        }
    }

    pub async fn dl_from_hash(&self, down: &ModDownload) -> Result<Option<Downloadable>> {
        if down.hash.is_empty() {
            Ok(None)
        } else {
            let fmt = match down.hash_format {
                HashFormat::Sha512 => "sha512",
                HashFormat::Sha1 => "sha1",
                _ => return Ok(None),
            };

            Ok(self
                .0
                .modrinth()
                .version_from_hash(&down.hash, fmt)
                .await
                .map(|ver| Downloadable::Modrinth {
                    id: ver.project_id.clone(),
                    version: ver.id.clone(),
                })
                .ok())
        }
    }

    pub fn dl_from_mod_update(&self, mod_update: Option<&ModUpdate>) -> Option<Downloadable> {
        if let Some(upd) = mod_update {
            if let Some(mr) = &upd.modrinth {
                Some(Downloadable::Modrinth {
                    id: mr.mod_id.clone(),
                    version: mr.version.clone(),
                })
            } else if let Some(cf) = &upd.curseforge {
                Some(Downloadable::CurseRinth {
                    id: cf.project_id.to_string(),
                    version: cf.file_id.to_string(),
                })
            } else {
                // TODO clarify
                self.0.warn("Unknown mod update".to_owned());
                None
            }
        } else {
            None
        }
    }

    pub async fn export_all(&self, output_dir: PathBuf) -> Result<()> {
        let progress_bar = self.0.multi_progress.add(ProgressBar::new_spinner());
        progress_bar.set_style(ProgressStyle::with_template("{spinner:.blue} {msg}")?);
        progress_bar.set_message("Converting mods...");
        progress_bar.enable_steady_tick(Duration::from_millis(250));

        let mut files_list = vec![];

        self.export_mods(&mut files_list, &output_dir).await?;

        progress_bar.set_message("Exporting configs...");

        self.export_configs(&mut files_list, &output_dir).await?;

        let index = PackIndex {
            files: files_list,
            hash_format: HashFormat::Sha256,
        };

        progress_bar.set_message("Saving index...");

        let mut f = File::create(output_dir.join("index.toml")).await?;
        let content = toml::to_string_pretty(&index)?;
        let index_hash = App::hash_sha256(&content);
        f.write_all(content.as_bytes()).await?;

        let pack = Pack {
            pack_format: "packwiz:1.1.0".to_owned(),
            name: self
                .0
                .var("MODPACK_NAME")
                .unwrap_or(self.0.server.name.clone()),
            versions: self.0.server.to_map(false),

            author: self.0.var("MODPACK_AUTHOR"),
            description: self.0.var("MODPACK_DESCRIPTION"),
            version: self.0.var("MODPACK_VERSION"),

            index: PackFile {
                file: "index.toml".to_owned(),
                hash: index_hash,
                hash_format: Some("sha256".to_owned()),
                metafile: false,
                preserve: false,
                alias: None,
            },
        };

        let mut f = File::create(output_dir.join("pack.toml")).await?;
        f.write_all(toml::to_string_pretty(&pack)?.as_bytes())
            .await?;

        progress_bar.finish_and_clear();
        self.0.success("Exported to packwiz successfully");

        if let Ok(u) = try_get_url(&output_dir.join("pack.toml")) {
            self.0.info("Exported pack URL:");
            self.0
                .log(format!("             https://raw.githack.com/{u}",));
        }

        Ok(())
    }

    pub async fn export_mods(
        &self,
        files_list: &mut Vec<PackFile>,
        output_dir: &Path,
    ) -> Result<()> {
        let pb = self
            .0
            .multi_progress
            .add(ProgressBar::new_spinner())
            .with_style(ProgressStyle::with_template(
                "{prefix:.blue.bold} {msg} [{wide_bar:.cyan/blue}] {pos}/{len}",
            )?)
            .with_prefix(ProgressPrefix::Exporting);

        tokio::fs::create_dir_all(output_dir.join("mods")).await?;
        for dl in self.0.server.mods.iter().progress_with(pb.clone()) {
            pb.set_message(dl.to_short_string());

            let m = self.to_mod(dl).await?;

            let filename = format!("{}.pw.toml", m.name);
            let path = output_dir.join("mods").join(&filename);

            let mut f = File::create(path).await?;

            let content = toml::to_string_pretty(&m)?;
            let hash = App::hash_sha256(&content);

            f.write_all(content.as_bytes()).await?;

            self.0.notify(Prefix::Exported, format!("mods/{filename}"));

            files_list.push(PackFile {
                file: format!("mods/{filename}"),
                hash,
                metafile: true,
                alias: None,
                hash_format: None,
                preserve: false,
            });
        }

        Ok(())
    }

    pub async fn export_configs(
        &self,
        files_list: &mut Vec<PackFile>,
        output_dir: &Path,
    ) -> Result<()> {
        let pb = self.0.multi_progress.add(
            ProgressBar::new_spinner()
                .with_style(ProgressStyle::with_template(
                    "{spinner:.blue} {prefix} {msg}",
                )?)
                .with_prefix(ProgressPrefix::Exporting),
        );
        pb.enable_steady_tick(Duration::from_millis(250));

        for entry in WalkDir::new(self.0.server.path.join("config")) {
            let entry = entry.map_err(|e| {
                anyhow!(
                    "Can't walk directory/file: {}",
                    &e.path().unwrap_or(Path::new("<unknown>")).display()
                )
            })?;

            if entry.file_type().is_dir() {
                continue;
            }

            let source = entry.path();
            let rel_path = diff_paths(source, self.0.server.path.join("config"))
                .ok_or(anyhow!("Cannot diff paths"))?;

            pb.set_message(rel_path.to_string_lossy().to_string());

            let source = self.0.server.path.join("config").join(&rel_path);
            let dest = output_dir.join(&rel_path);

            tokio::fs::create_dir_all(dest.parent().unwrap())
                .await
                .context("Creating parent directory")?;

            let mut source_file = File::open(&source).await?;
            let mut dest_file = File::create(&dest).await?;

            let hash = App::copy_with_hashing(
                &mut source_file,
                &mut dest_file,
                App::create_hasher("sha256"),
            )
            .await?;

            files_list.push(PackFile {
                file: rel_path.to_string_lossy().into_owned(),
                hash,
                metafile: false,
                alias: None,
                hash_format: None,
                preserve: false,
            });
        }

        pb.finish_and_clear();

        Ok(())
    }

    pub async fn to_mod(&self, dl: &Downloadable) -> Result<Mod> {
        let resolved = dl.resolve_source(self.0).await?;

        let mut m = self.resolved_to_mod(&resolved).await?;
        m.update = Self::get_mod_update(dl);

        Ok(m)
    }

    pub fn get_mod_update(dl: &Downloadable) -> Option<ModUpdate> {
        match dl {
            Downloadable::Modrinth { id, version } => Some(ModUpdate {
                modrinth: Some(rpackwiz::model::ModrinthModUpdate {
                    mod_id: id.clone(),
                    version: version.clone(),
                }),
                curseforge: None,
            }),
            // too much work, not worth it
            // id is u64 in toml, idk why
            //Downloadable::CurseRinth
            _ => None,
        }
    }

    pub async fn resolved_to_mod(&self, resolved_file: &ResolvedFile) -> Result<Mod> {
        let (hash_format, hash) = self.0.hash_resolved_file(resolved_file).await?;

        Ok(Mod {
            filename: resolved_file.filename.clone(),
            name: resolved_file
                .filename
                .strip_suffix(".jar")
                .unwrap_or(&resolved_file.filename)
                .to_string(),
            download: ModDownload {
                url: Some(resolved_file.url.clone()),
                hash,
                hash_format: match hash_format.as_str() {
                    "sha1" => HashFormat::Sha1,
                    "sha256" => HashFormat::Sha256,
                    "sha512" => HashFormat::Sha512,
                    "murmur2" => HashFormat::Curseforge,
                    _ => HashFormat::Md5,
                },
                mode: DownloadMode::Url,
            },
            ..Default::default()
        })
    }
}
