use anyhow::Result;
use clap::{Parser, Subcommand};
use mcman_core::ctx::Context;
use mcman_sources::sources::modrinth::ModrinthAPI;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Test,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let ctx = Context::new()?;

    let api = ModrinthAPI {
        // url: "https://staging-api.modrinth.com".to_string(),
        url: "https://api.modrinth.com".to_string(),
    };

    let sodium = api.get_project(&ctx, "sodium").await?;

    println!("{sodium:?}");

    Ok(())
}
