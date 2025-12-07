mod models;
use mcman_core::ctx::Context;
use miette::Result;
pub use models::*;

pub const GITHUB_API_URL: &str = "https://api.github.com";
pub const GITHUB_API_VERSION: &str = "2022-11-28";

pub struct GitHubAPI(pub String);

impl Default for GitHubAPI {
    fn default() -> Self {
        Self(GITHUB_API_URL.to_owned())
    }
}

impl GitHubAPI {
    pub async fn fetch_releases(&self, ctx: &Context, repo: &str) -> Result<Vec<GithubRelease>> {
        ctx.fetch_json(format!("{}/repos/{repo}/releases", self.0))
            .await
    }

    pub fn resolve_asset_url(&self, repo: &str, release: &GithubRelease, asset: &GithubAsset) -> String {
        format!(
            "https://github.com/{repo}/releases/download/{}/{}",
            release.tag_name, asset.name
        )
    }
}
