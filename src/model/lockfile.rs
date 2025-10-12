use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::BufWriter,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::app::ResolvedFile;

use super::Downloadable;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Lockfile {
    #[serde(skip)]
    pub path: PathBuf,

    pub plugins: Vec<(Downloadable, ResolvedFile)>,
    pub mods: Vec<(Downloadable, ResolvedFile)>,

    pub server_vars: HashMap<String, String>,
    pub nw_vars: HashMap<String, String>,

    pub files: Vec<BootstrappedFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BootstrappedFile {
    pub path: PathBuf,
    pub date: SystemTime,
}

impl Lockfile {
    pub fn get_lockfile(output_dir: &Path) -> Result<Self> {
        if output_dir.join(".mcman.lock").exists() {
            Ok(Self::load_from(&output_dir.join(".mcman.lock"))?)
        } else {
            Ok(Self {
                path: output_dir.join(".mcman.lock"),
                ..Default::default()
            })
        }
    }

    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let data = read_to_string(path)?;
        let mut nw: Self = serde_json::from_str(&data)?;
        nw.path.clone_from(path);
        Ok(nw)
    }

    pub fn save(&self) -> Result<()> {
        let writer = BufWriter::new(File::create(&self.path)?);

        Ok(serde_json::to_writer_pretty(writer, &self)?)
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./.mcman.lock"),
            plugins: vec![],
            mods: vec![],
            files: vec![],
            server_vars: HashMap::default(),
            nw_vars: HashMap::default(),
        }
    }
}
