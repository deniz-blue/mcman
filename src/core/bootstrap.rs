use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use pathdiff::diff_paths;
use tokio::fs;
use walkdir::WalkDir;

use crate::{model::BootstrappedFile, util::dollar_repl};

use super::BuildContext;

impl BuildContext<'_> {
    pub async fn bootstrap_files(&mut self) -> Result<()> {
        self.app.print_job("Bootstrapping...");

        self.app.ci("::group::Bootstrapping");

        let pb = self.app.multi_progress.add(
            ProgressBar::new_spinner()
                .with_style(ProgressStyle::with_template(
                    "{spinner:.blue} {prefix} {msg}",
                )?)
                .with_prefix("Bootstrapping"),
        );
        pb.enable_steady_tick(Duration::from_millis(250));

        /* let lockfile_entries = self
        .lockfile
        .files
        .iter()
        .map(|e| (e.path.clone(), e.date))
        .collect::<HashMap<_, _>>(); */

        // wtf i have to clone it sorry null
        if let Some(nw) = self.app.network.clone() {
            self.bootstrap_folder(nw.path.join("groups").join("global").join("config"))
                .await?;

            if self.app.server.name == nw.proxy {
                for group_name in &nw.proxy_groups {
                    self.bootstrap_folder(nw.path.join("groups").join(group_name).join("config"))
                        .await?;
                }
            }

            if let Some(entry) = nw.servers.get(&self.app.server.name) {
                for group_name in &entry.groups {
                    self.bootstrap_folder(nw.path.join("groups").join(group_name).join("config"))
                        .await?;
                }
            }
        }

        self.bootstrap_folder(self.app.server.path.join("config"))
            .await?;

        pb.disable_steady_tick();
        pb.finish_and_clear();
        self.app.success("Bootstrapping complete");

        self.app.ci("::endgroup::");

        Ok(())
    }

    pub async fn bootstrap_folder(&mut self, from_path: PathBuf) -> Result<()> {
        if !from_path.exists() {
            self.app.dbg(format!(
                "skipped bootstrapping {} because it doesnt exist",
                from_path.display()
            ));
            return Ok(());
        }

        for entry in WalkDir::new(&from_path) {
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
            let diffed_paths =
                diff_paths(source, &from_path).ok_or(anyhow!("Cannot diff paths"))?;

            //pb.set_message(diffed_paths.to_string_lossy().to_string());

            self.bootstrap_file(
                &source.to_path_buf(),
                &diffed_paths,
                None, /* lockfile_entries.get(&diffed_paths) */
            )
            .await
            .context(format!(
                "Bootstrapping file:
                - Entry: {}
                - Relative: {}",
                entry.path().display(),
                diffed_paths.display()
            ))?;
        }

        Ok(())
    }

    pub fn should_bootstrap_file(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        let bootstrap_exts = [
            "properties",
            "txt",
            "yaml",
            "yml",
            "conf",
            "config",
            "toml",
            "json",
            "json5",
            "secret",
        ];

        bootstrap_exts.contains(&ext)
            || self
                .app
                .server
                .options
                .bootstrap_exts
                .iter()
                .any(|s| s.as_str() == ext)
    }

    pub async fn bootstrap_file(
        &mut self,
        full_path: &PathBuf,
        rel_path: &PathBuf,
        cache: Option<&SystemTime>,
    ) -> Result<()> {
        let pretty_path = rel_path.display();

        let source = full_path;
        let dest = self.output_dir.join(rel_path);

        let metadata = fs::metadata(&source).await.context(format!(
            "Getting metadata
                - File: {}
                - Relative path: {pretty_path}
                - Destination: {}",
            source.display(),
            dest.display(),
        ))?;

        let modified = metadata.modified();

        if self.force || {
            if let Some(time) = cache {
                if let Ok(source_time) = modified {
                    &source_time > time
                } else {
                    true
                }
            } else {
                true
            }
        } {
            fs::create_dir_all(dest.parent().unwrap())
                .await
                .context("Creating parent directory")?;

            if self.should_bootstrap_file(rel_path) {
                let config_contents = fs::read_to_string(&source).await.context(format!(
                    "Reading from '{}' ; [{pretty_path}]",
                    source.display()
                ))?;
                let permissions = fs::metadata(&source)
                    .await
                    .context(format!(
                        "Getting metadata of '{}' ; [{pretty_path}]",
                        source.display()
                    ))?
                    .permissions();

                let bootstrapped_contents = self.bootstrap_content(&config_contents);

                fs::write(&dest, bootstrapped_contents)
                    .await
                    .context(format!("Writing to '{}' ; [{pretty_path}]", dest.display()))?;
                fs::set_permissions(&dest, permissions)
                    .await
                    .context(format!(
                        "Setting permissions for '{}' ; [{pretty_path}]",
                        dest.display()
                    ))?;
            } else {
                // ? idk why but read_to_string and fs::write works with 'dest' but fs::copy doesnt
                fs::copy(&source, &dest).await.context(format!(
                    "Copying '{}' to '{}' ; [{pretty_path}]",
                    source.display(),
                    dest.display()
                ))?;
            }

            self.app.log_dev(format!("=> {pretty_path}"));
        } else {
            self.app.log_dev(format!("   {pretty_path}"));
        }

        if let Ok(source_time) = modified {
            self.new_lockfile.files.push(BootstrappedFile {
                path: rel_path.clone(),
                date: source_time,
            });
        } else {
            self.app.warn("File metadata not supported");
        }

        Ok(())
    }

    pub fn bootstrap_content(&self, content: &str) -> String {
        dollar_repl(content, |k| {
            let k = k.trim();

            let (k, def) = if let Some((k, def)) = k.split_once(':') {
                (k.trim(), Some(def.trim().to_owned()))
            } else {
                (k, None)
            };

            self.app.var(k).or(def)
        })
    }
}
