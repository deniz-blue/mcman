use clap::Parser;
use mcman::cli::Cli;
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    Cli::parse().run().await
}
