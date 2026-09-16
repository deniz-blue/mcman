use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use miette::{bail, IntoDiagnostic, Result, WrapErr};

use crate::{
    actions::all_in_store,
    core::AppContext,
    lockfile::{Artifact, LockedPackage, LockedTarget},
    package::{
        build::tasks::{BuildTaskContext, BuildTaskRunner},
        source::PackageSource,
        Package,
    },
    plan::TargetPlan,
    store::ObjectKey,
};

pub async fn build_packages(
    ctx: &AppContext,
    plan: &TargetPlan,
    target: &mut LockedTarget,
) -> Result<()> {
    let mut built = Vec::new();

    for directory in &plan.directories {
        let root = directory.path.clone().unwrap_or_default();

        for package in &directory.packages {
            let held = target
                .packages
                .iter()
                .find(|held| held.name == package.label);

            built.push(build_package(ctx, &root, package, held).await?);
        }
    }

    target.packages = built;

    Ok(())
}

async fn build_package(
    ctx: &AppContext,
    directory: &Path,
    package: &Package,
    held: Option<&LockedPackage>,
) -> Result<LockedPackage> {
    let workspace = ctx.store.tmp()?;
    let contents = fetch_sources(ctx, package, workspace.path()).await?;
    let identity = identity_of(package, &contents);

    if let Some(held) = held {
        if held.identity == identity && all_in_store(ctx, &held.artifacts).await? {
            return Ok(held.clone());
        }
    }

    if let Some(build) = &package.build {
        let context = BuildTaskContext {
            path: workspace.path().to_owned(),
        };

        build
            .run(&context)
            .await
            .wrap_err_with(|| format!("building `{}`", package.label))?;
    }

    Ok(LockedPackage {
        name: package.label.clone(),
        identity,
        artifacts: collect(ctx, directory, package, workspace.path()).await?,
    })
}

async fn fetch_sources(
    ctx: &AppContext,
    package: &Package,
    workspace: &Path,
) -> Result<Vec<String>> {
    let mut contents = Vec::new();

    for source in &package.sources {
        match source {
            PackageSource::Download(download) => {
                let key = download.run(ctx).await?;
                let destination = workspace.join(download.destination());

                if let Some(parent) = destination.parent() {
                    tokio::fs::create_dir_all(parent).await.into_diagnostic()?;
                }

                tokio::fs::copy(ctx.store.object_path(&key), &destination)
                    .await
                    .into_diagnostic()
                    .wrap_err_with(|| format!("could not write `{}`", destination.display()))?;

                make_writable(&destination).await?;
                contents.push(key.to_hex());
            }
            PackageSource::Git(git) => contents.push(git.run(workspace).await?),
        }
    }

    Ok(contents)
}

fn identity_of(package: &Package, contents: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(package.to_kdl_string().as_bytes());

    for content in contents {
        hasher.update(content.as_bytes());
        hasher.update(b"\n");
    }

    hasher.finalize().to_hex().to_string()
}

async fn collect(
    ctx: &AppContext,
    directory: &Path,
    package: &Package,
    workspace: &Path,
) -> Result<Vec<Artifact>> {
    let mut stored: HashMap<PathBuf, (String, u64)> = HashMap::new();
    let mut artifacts = Vec::new();

    for artifact in &package.artifacts {
        let (hash, size) = match stored.get(&artifact.from) {
            Some(found) => found.clone(),
            None => {
                let source = workspace.join(&artifact.from);

                if !tokio::fs::try_exists(&source).await.into_diagnostic()? {
                    bail!(
                        "`{}` left no `{}` behind",
                        package.label,
                        artifact.from.display()
                    );
                }

                let size = tokio::fs::metadata(&source)
                    .await
                    .into_diagnostic()
                    .wrap_err_with(|| format!("could not measure `{}`", source.display()))?
                    .len();

                let key: ObjectKey = ctx.store.put(&source).await?;
                let found = (key.to_hex(), size);
                stored.insert(artifact.from.clone(), found.clone());

                found
            }
        };

        artifacts.push(Artifact {
            path: directory.join(artifact.destination()),
            hash,
            size,
        });
    }

    Ok(artifacts)
}

#[cfg(unix)]
async fn make_writable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not unlock `{}`", path.display()))
}

#[cfg(not(unix))]
async fn make_writable(path: &Path) -> Result<()> {
    let mut permissions = tokio::fs::metadata(path)
        .await
        .into_diagnostic()?
        .permissions();
    permissions.set_readonly(false);

    tokio::fs::set_permissions(path, permissions)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not unlock `{}`", path.display()))
}
