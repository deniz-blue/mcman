use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use indicatif::ProgressBar;
use pathdiff::diff_paths;
use walkdir::WalkDir;
use zip::write::FileOptions;

use crate::app::{App, Prefix};

pub struct WorldsAPI<'a>(pub &'a App);

impl WorldsAPI<'_> {
    pub fn unpack(&self, world: &str) -> Result<()> {
        let spinner = self
            .0
            .multi_progress
            .add(ProgressBar::new_spinner())
            .with_message(format!("Unzipping world '{world}'..."));
        spinner.enable_steady_tick(Duration::from_millis(250));
        let zip_path = self
            .0
            .server
            .path
            .join("worlds")
            .join(format!("{world}.zip"));

        if !zip_path.exists() {
            bail!("worlds/{world}.zip doesnt exist");
        }

        self.unzip(&zip_path, &self.0.server.path.join("server").join(world))?;

        spinner.finish();
        self.0.notify(Prefix::Unpacked, format!("world {world}"));

        Ok(())
    }

    #[allow(clippy::unused_self)]
    pub fn unzip(&self, zip_archive_path: &PathBuf, output: &Path) -> Result<()> {
        let file = File::open(zip_archive_path)
            .context(format!("Opening zip file: {}", zip_archive_path.display()))?;
        let mut archive = zip::ZipArchive::new(file).context("Opening zip archive")?;

        let files = archive
            .file_names()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        for filename in files {
            if filename.ends_with('/') {
                continue; // folder
            }

            let mut zip_file = archive.by_name(&filename)?;
            let target_path = output.join(filename);

            fs::create_dir_all(target_path.parent().unwrap())?;
            let mut target_file = File::create(&target_path)?;
            io::copy(&mut zip_file, &mut target_file)?;
        }

        Ok(())
    }

    pub fn pack(&self, world: &str) -> Result<()> {
        let spinner = self
            .0
            .multi_progress
            .add(ProgressBar::new_spinner())
            .with_message(format!("Zipping world '{world}'..."));

        spinner.enable_steady_tick(Duration::from_millis(250));

        let input_path = self.0.server.path.join("server").join(world);
        let output_path = self
            .0
            .server
            .path
            .join("worlds")
            .join(format!("{world}.zip"));
        fs::create_dir_all(self.0.server.path.join("worlds"))?;
        let output_file = File::create(output_path)?;

        let mut zip = zip::ZipWriter::new(output_file);

        for entry in WalkDir::new(&input_path) {
            let entry = entry.map_err(|e| {
                anyhow!(
                    "Can't walk directory/file: {}",
                    &e.path().unwrap_or(Path::new("<unknown>")).display()
                )
            })?;

            let source = entry.path();
            let diffed_paths =
                diff_paths(source, &input_path).ok_or(anyhow!("Cannot diff paths"))?;

            if entry.file_type().is_dir() {
                zip.add_directory(diffed_paths.to_string_lossy(), FileOptions::default())?;
                continue;
            }

            spinner.set_message(diffed_paths.to_string_lossy().to_string());

            zip.start_file(diffed_paths.to_string_lossy(), FileOptions::default())?;

            let mut input_file = File::open(source)?;

            io::copy(&mut input_file, &mut zip)?;
        }

        spinner.finish();
        self.0.notify(Prefix::Packed, format!("world {world}"));

        Ok(())
    }
}
