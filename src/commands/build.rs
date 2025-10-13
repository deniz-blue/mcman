use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{app::App, core::BuildContext, model::Lockfile};

#[derive(clap::Args)]
pub struct BuildArgs {
    /// The output directory for the server
    #[arg(short, long, value_name = "file")]
    output: Option<PathBuf>,
    /// Skip some stages
    #[arg(short, long, value_name = "stages")]
    skip: Vec<String>,
    #[arg(long)]
    /// Don't skip downloading already downloaded jars
    force: bool,
}

impl BuildArgs {
    pub fn create_build_context(self, app: &mut App) -> Result<BuildContext<'_>> {
        let default_output = app.server.path.join("server");
        let output_dir = self.output.unwrap_or(default_output);

        std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

        Ok(BuildContext {
            app,
            force: self.force,
            skip_stages: self.skip,
            output_dir,
            lockfile: Lockfile::default(),
            new_lockfile: Lockfile::default(),
            _server_process: None,
        })
    }
}

pub async fn run(mut app: App, args: BuildArgs) -> Result<()> {
    let mut ctx = args.create_build_context(&mut app)?;

    ctx.build_all().await?;

    Ok(())
}
