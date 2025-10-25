use anyhow::Result;
use mcman_core::ctx::Context;

use crate::sources::modrinth::models::project::ModrinthProject;

pub mod models;

pub struct ModrinthAPI {
    pub url: String,
}

impl ModrinthAPI {
    pub async fn get_project(&self, ctx: &Context, id: &str) -> Result<ModrinthProject> {
        ctx.fetch_json(format!("{}/v2/project/{id}", self.url)).await
    }
}


