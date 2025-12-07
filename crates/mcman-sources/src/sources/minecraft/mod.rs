use mcman_core::ctx::Context;
use miette::Result;

mod models;
pub use models::*;

pub const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

pub struct PistonMeta(pub String);

impl Default for PistonMeta {
    fn default() -> Self {
        Self(VERSION_MANIFEST_URL.to_owned())
    }
}

impl PistonMeta {
    pub async fn fetch_manifest(&self, ctx: &Context) -> Result<VersionManifest> {
        ctx.fetch_json(&self.0).await
    }
}

impl VersionIndex {
    pub async fn fetch(&self, ctx: &Context) -> Result<VersionInfo> {
        ctx.fetch_json(&self.url).await
    }
}
