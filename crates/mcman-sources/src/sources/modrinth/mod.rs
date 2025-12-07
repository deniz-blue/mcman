use mcman_core::ctx::Context;
use miette::Result;

mod models;
pub use models::*;

pub const MODRINTH_API_URL: &str = "";

pub struct ModrinthAPI {
    pub base_url: String,
    pub api_key: Option<String>,
}

impl Default for ModrinthAPI {
    fn default() -> Self {
        Self {
            base_url: MODRINTH_API_URL.to_owned(),
            api_key: None,
        }
    }
}

impl ModrinthAPI {
    pub async fn get_project(&self, ctx: &Context, id: &str) -> Result<ModrinthProject> {
        ctx.fetch_json(format!("{}/v2/project/{id}", self.base_url))
            .await
    }
}
