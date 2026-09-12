use std::path::{Path, PathBuf};

use miette::{Diagnostic, IntoDiagnostic, Result};
use thiserror::Error;

#[derive(Debug, confique::Config)]
pub struct Config {
    pub store: Option<PathBuf>,
}

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    #[error("no home directory to place mcman's files under")]
    #[diagnostic(code(mcman::no_home))]
    NoHome,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = dirs::config_dir()
            .ok_or(ConfigError::NoHome)?
            .join("mcman")
            .join("config.toml");

        confique::Config::builder()
            .file(path)
            .load()
            .into_diagnostic()
    }

    pub fn store_path(&self, flag: Option<&Path>) -> Result<PathBuf> {
        if let Some(path) = flag {
            return Ok(path.to_owned());
        }

        if let Some(path) = &self.store {
            return Ok(path.clone());
        }

        Ok(dirs::cache_dir().ok_or(ConfigError::NoHome)?.join("mcman"))
    }
}
