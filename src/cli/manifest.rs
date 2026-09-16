use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Args;
use miette::{bail, Diagnostic, IntoDiagnostic, NamedSource, Result, WrapErr};
use thiserror::Error;

use crate::{
    actions,
    cli::ManifestArgs,
    config::Config,
    core::{location::Location, AppContext},
    lockfile::{Lockfile, LockfileMeta},
    manifest::Manifest,
    plan::{self, Plan},
    resolve,
    store::Store,
};

const LOCKFILE_VERSION: u64 = 1;

const MANIFEST_NAME: &str = "mcman.kdl";
const LOCKFILE_NAME: &str = "mcman.lock";

const INIT_TEMPLATE: &str = r#"target smp

platform minecraft version="1.21.1"
platform paper

dir plugins {
    use modrinth luckperms
}
"#;

#[derive(Debug, Error, Diagnostic)]
pub enum ManifestLookupError {
    #[error("no `{MANIFEST_NAME}` found in `{}` or any directory above it", .from.display())]
    #[diagnostic(
        code(mcman::no_manifest),
        help("Run `mcman init` to create one, or name it with `-f`.")
    )]
    NotFound { from: PathBuf },

    #[error("`{}` already exists", .path.display())]
    #[diagnostic(code(mcman::manifest_exists))]
    AlreadyExists { path: PathBuf },
}

pub fn find_manifest(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_owned());
    }

    let from = std::env::current_dir().into_diagnostic()?;

    for directory in from.ancestors() {
        let candidate = directory.join(MANIFEST_NAME);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(ManifestLookupError::NotFound { from }.into())
}

struct Loaded {
    path: PathBuf,
    plan: Plan,
}

async fn load(args: &ManifestArgs) -> Result<Loaded> {
    let path = find_manifest(args.manifest.as_deref())?;
    let text = tokio::fs::read_to_string(&path)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("reading manifest {}", path.display()))?;

    let name = path.to_string_lossy();
    let manifest = Manifest::parse(&name, &text)?;
    let plan = plan::from_manifest(&manifest).map_err(|error| {
        miette::Report::new(error).with_source_code(NamedSource::new(name, text))
    })?;

    Ok(Loaded { path, plan })
}

fn lockfile_path(manifest: &Path) -> PathBuf {
    manifest.with_file_name(LOCKFILE_NAME)
}

async fn load_lockfile(manifest: &Path) -> Result<Lockfile> {
    let path = lockfile_path(manifest);

    match tokio::fs::read_to_string(&path).await {
        Ok(text) => Lockfile::parse(&path.to_string_lossy(), &text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Lockfile::default()),
        Err(error) => Err(error)
            .into_diagnostic()
            .wrap_err_with(|| format!("reading lockfile {}", path.display())),
    }
}

async fn write_lockfile(manifest: &Path, lockfile: &Lockfile) -> Result<()> {
    let path = lockfile_path(manifest);

    tokio::fs::write(&path, lockfile.to_kdl())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("writing lockfile {}", path.display()))
}

async fn resolve_all(
    ctx: &AppContext,
    manifest: &Path,
    plan: &Plan,
    held: &Lockfile,
    reuse: impl Fn(&str) -> bool,
) -> Result<Lockfile> {
    let location = Location::Path(manifest.to_owned());
    let mut targets = Vec::new();

    for target in &plan.targets {
        let name = target.target.name.as_str();
        let locked = held
            .targets
            .iter()
            .find(|locked| locked.name == name)
            .filter(|_| reuse(name));

        targets.push(resolve::resolve_target(ctx, &location, target, locked).await?);
    }

    Ok(Lockfile {
        meta: LockfileMeta {
            version: LOCKFILE_VERSION,
            generated: jiff::Timestamp::now().to_string(),
        },
        targets,
    })
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[command(flatten)]
    pub manifest: ManifestArgs,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub locked: bool,
    #[arg(long)]
    pub offline: bool,
    #[arg(long)]
    pub force: bool,
    pub targets: Vec<String>,
}

impl BuildArgs {
    pub async fn run(self, store: Option<&Path>) -> Result<()> {
        let loaded = load(&self.manifest).await?;

        for warning in &loaded.plan.warnings {
            eprintln!("warning: {warning}");
        }

        let held = load_lockfile(&loaded.path).await?;
        let changes = held.changes_needed_for(&loaded.plan);

        if self.locked && !changes.is_empty() {
            for change in &changes {
                eprintln!("  {change}");
            }
            bail!("the lockfile does not cover the manifest, and `--locked` was given");
        }

        if self.dry_run {
            for change in &changes {
                println!("{change}");
            }
            return Ok(());
        }

        let config = Config::load()?;
        let store = Store::open(config.store_path(store)?).await?;
        let ctx = AppContext::new(Arc::new(store));

        let mut lockfile = if changes.is_empty() {
            held
        } else {
            resolve_all(&ctx, &loaded.path, &loaded.plan, &held, |_| true).await?
        };

        let manifest_dir = loaded.path.parent().unwrap_or(Path::new("."));

        for planned in &loaded.plan.targets {
            let name = &planned.target.name;

            if !self.targets.is_empty() && !self.targets.contains(name) {
                continue;
            }

            let Some(target) = lockfile.targets.iter_mut().find(|held| &held.name == name) else {
                bail!("`{name}` is missing from the lockfile");
            };

            actions::build::build_packages(&ctx, planned, target).await?;

            let root = manifest_dir.join(&target.path);
            actions::materialize::materialize(&ctx, &root, target).await?;
        }

        write_lockfile(&loaded.path, &lockfile).await?;

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    #[command(flatten)]
    pub manifest: ManifestArgs,
    pub packages: Vec<String>,
}

impl UpdateArgs {
    pub async fn run(self, store: Option<&Path>) -> Result<()> {
        let loaded = load(&self.manifest).await?;
        let held = load_lockfile(&loaded.path).await?;

        let config = Config::load()?;
        let store = Store::open(config.store_path(store)?).await?;
        let ctx = AppContext::new(Arc::new(store));

        let keep =
            |name: &str| !self.packages.is_empty() && !self.packages.contains(&name.to_owned());
        let lockfile = resolve_all(&ctx, &loaded.path, &loaded.plan, &held, keep).await?;
        write_lockfile(&loaded.path, &lockfile).await?;

        for change in held.changes_needed_for(&loaded.plan) {
            println!("{change}");
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct ExplainArgs {
    #[command(flatten)]
    pub manifest: ManifestArgs,
    pub targets: Vec<String>,
}

impl ExplainArgs {
    pub async fn run(self) -> Result<()> {
        let loaded = load(&self.manifest).await?;

        for target in &loaded.plan.targets {
            if !self.targets.is_empty() && !self.targets.contains(&target.target.name) {
                continue;
            }

            println!("target {}", target.target.name);

            for platform in &target.platforms {
                println!("  platform {}", platform.value);
            }

            for runtime in &target.runtimes {
                println!("  runtime {}", runtime.value);
            }

            for include in &target.includes {
                println!("  include {}", include.value);
            }

            for directory in &target.directories {
                match &directory.path {
                    Some(path) => println!("  dir {}", path.display()),
                    None => println!("  dir ."),
                }

                for addon in &directory.addons {
                    println!("    use {}", addon.value);
                }

                for package in &directory.packages {
                    println!("    package {}", package.label);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct InitArgs {
    pub path: Option<PathBuf>,
}

impl InitArgs {
    pub async fn run(self) -> Result<()> {
        let path = match self.path {
            Some(path) if path.is_dir() => path.join(MANIFEST_NAME),
            Some(path) => path,
            None => PathBuf::from(MANIFEST_NAME),
        };

        if path.exists() {
            return Err(ManifestLookupError::AlreadyExists { path }.into());
        }

        tokio::fs::write(&path, INIT_TEMPLATE)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("writing {}", path.display()))?;

        println!("wrote {}", path.display());
        Ok(())
    }
}
