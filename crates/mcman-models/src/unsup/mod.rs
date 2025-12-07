use serde::{Deserialize, Serialize};

mod root;
pub use root::*;
mod update;
pub use update::*;
mod bootstrap;
pub use bootstrap::*;
mod hash_format;
pub use hash_format::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub code: u128,
}
