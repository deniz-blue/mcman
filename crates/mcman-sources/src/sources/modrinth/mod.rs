use mcman_core::ctx::Context;
use miette::Result;

mod models;
pub use models::*;

pub const MODRINTH_API_URL: &str = "https://api.modrinth.com";

pub struct ModrinthAPI(pub String);

impl Default for ModrinthAPI {
    fn default() -> Self {
        Self(MODRINTH_API_URL.to_owned())
    }
}

impl ModrinthAPI {
    pub async fn get_project(&self, ctx: &Context, id: &str) -> Result<ModrinthProject> {
        ctx.fetch_json(format!("{}/v2/project/{id}", self.0))
            .await
    }
}
