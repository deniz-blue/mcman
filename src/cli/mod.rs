use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use miette::Result;

mod manifest;

pub use manifest::{BuildArgs, ExplainArgs, InitArgs, UpdateArgs};

#[derive(Debug, Parser)]
#[command(name = "mcman", version)]
pub struct Cli {
    #[arg(long, env = "MCMAN_STORE", global = true)]
    pub store: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Build(BuildArgs),
    Update(UpdateArgs),
    Explain(ExplainArgs),
    Init(InitArgs),
}

#[derive(Debug, Args)]
pub struct ManifestArgs {
    #[arg(short = 'f', long)]
    pub manifest: Option<PathBuf>,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Command::Build(args) => args.run(self.store.as_deref()).await,
            Command::Update(args) => args.run().await,
            Command::Explain(args) => args.run().await,
            Command::Init(args) => args.run().await,
        }
    }
}
