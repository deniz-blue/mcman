use anyhow::Result;
use app::BaseApp;
use clap::Parser;

mod app;
mod commands;
mod core;
mod hot_reload;
mod interop;
mod model;
mod sources;
mod util;

use std::env;

#[derive(clap::Parser)]
#[clap(name = "mcman", version)]
#[command(author = "ParadigmMC", color = clap::ColorChoice::Always)]
#[command(about = "Powerful Minecraft Server Manager CLI")]
#[command(after_help = "To start building servers, try 'mcman init'")]
#[command(subcommand_required = true, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize a new mcman server
    Init(commands::init::Args),

    /// Build using server.toml configuration
    Build(commands::build::BuildArgs),
    /// Test the server (stops it when it ends startup)
    Run(commands::run::RunArgs),
    /// Start a development session
    Dev(commands::dev::DevArgs),

    /// Add a plugin/mod/datapack
    #[command(subcommand)]
    Add(commands::add::Commands),
    /// Pull files from server/ to config/
    Pull(commands::pull::Args),
    /// Helpers for setting up the environment
    #[command(subcommand)]
    Env(commands::env::Commands),
    /// Pack or unpack a world
    #[command(subcommand, visible_alias = "w")]
    World(commands::world::Commands),

    /// Importing tools
    #[command(subcommand, visible_alias = "i")]
    Import(commands::import::Commands),
    /// Exporting tools
    #[command(subcommand)]
    Export(commands::export::Commands),
    /// Update markdown files with server info
    #[command(visible_alias = "md")]
    Markdown,

    /// Download a downloadable
    #[command(visible_alias = "dl")]
    Download(commands::download::Args),
    /// Cache management commands
    #[command(subcommand)]
    Cache(commands::cache::Commands),
    /// Show info about the server in console
    Info,
    /// Show version information
    #[command(visible_alias = "v")]
    Version(commands::version::Args),

    /// Eject - remove everything related to mcman
    #[command(hide = true)]
    Eject,

    /// Generate autocompletion for shell
    /// (requires feature "autocomplete")
    #[cfg(feature = "autocomplete")]
    Completions(commands::completions::CompletionArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("CI").is_ok_and(|s| s.as_str() == "true") {
        println!("::endgroup::");
    }

    let args = Cli::parse();

    #[cfg(feature = "autocomplete")]
    if let Commands::Completions(args) = &args.command {
        commands::completions::run(args)?;
    }

    let base_app = BaseApp::new()?;

    match args.command {
        Commands::Init(args) => commands::init::run(base_app, args).await,
        Commands::Cache(subcommands) => commands::cache::run(subcommands),
        Commands::Version(args) => commands::version::run(base_app, args).await,
        c => {
            let mut app = base_app.upgrade()?;

            match c {
                // Build
                Commands::Build(args) => commands::build::run(app, args).await,
                Commands::Run(args) => commands::run::run(app, args).await,
                Commands::Dev(args) => commands::dev::run(app, args).await,

                // Management
                Commands::Add(commands) => commands::add::run(app, commands).await,
                Commands::Import(subcommands) => commands::import::run(app, subcommands).await,
                Commands::Export(commands) => commands::export::run(app, commands).await,
                Commands::Markdown => commands::markdown::run(app).await,
                Commands::World(commands) => commands::world::run(&mut app, commands),
                Commands::Pull(args) => commands::pull::run(&app, args),
                Commands::Env(commands) => commands::env::run(&app, commands),
                Commands::Eject => commands::eject::run(&app),

                // Utils
                Commands::Info => commands::info::run(&app),
                Commands::Download(args) => commands::download::run(app, args).await,

                _ => unreachable!(),
            }
        }
    }
}
