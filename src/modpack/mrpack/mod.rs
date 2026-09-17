use std::path::{Path, PathBuf};

use crate::modpack::Side;

pub mod error;
pub mod index;
pub mod reader;
pub mod writer;

pub use error::MrpackError;
pub use index::{
    MrpackEnv, MrpackFile, MrpackIndex, MrpackSupport, FORMAT_VERSION, GAME, INDEX_NAME,
};
pub use reader::{MrpackOverride, MrpackReader};
pub use writer::{MrpackHeader, MrpackWriter};

const SHARED_OVERRIDES: &str = "overrides";
const CLIENT_OVERRIDES: &str = "client-overrides";
const SERVER_OVERRIDES: &str = "server-overrides";

pub fn override_directory(side: Option<Side>) -> &'static str {
    match side {
        None => SHARED_OVERRIDES,
        Some(Side::Client) => CLIENT_OVERRIDES,
        Some(Side::Server) => SERVER_OVERRIDES,
    }
}

fn override_path(name: &Path, side: Option<Side>) -> Option<(PathBuf, bool)> {
    if let Ok(path) = name.strip_prefix(SHARED_OVERRIDES) {
        return Some((path.to_owned(), false));
    }

    name.strip_prefix(override_directory(Some(side?)))
        .ok()
        .map(|path| (path.to_owned(), true))
}
