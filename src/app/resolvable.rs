use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::App;

pub trait Resolvable {
    async fn resolve_source(&self, app: &App) -> Result<ResolvedFile>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFile {
    pub url: String,
    pub filename: String,
    pub cache: CacheStrategy,
    pub size: Option<u64>,
    pub hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum CacheStrategy {
    File {
        namespace: Cow<'static, str>,
        path: String,
    },
    Indexed {
        index_path: String,
        key: String,
        value: String,
    },
    #[default]
    None,
}
