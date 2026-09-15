use std::path::Path;

use miette::{Context, IntoDiagnostic, Result};

use crate::{
    core::AppContext,
    lockfile::{Artifact, Locked, LockedTarget},
    store::ObjectKey,
};

pub async fn materialize(ctx: &AppContext, root: &Path, target: &mut LockedTarget) -> Result<()> {
    for addon in &mut target.addons {
        place(ctx, root, addon).await?;
    }

    for include in &mut target.includes {
        place(ctx, root, include).await?;
    }

    for package in &target.packages {
        for artifact in &package.artifacts {
            place_artifact(ctx, root, artifact).await?;
        }
    }

    Ok(())
}

async fn place<D>(ctx: &AppContext, root: &Path, entry: &mut Locked<D>) -> Result<()> {
    if entry.artifacts.is_empty() || !all_in_store(ctx, &entry.artifacts).await? {
        entry.artifacts = fetch(ctx, entry).await?;
    }

    for artifact in &entry.artifacts {
        place_artifact(ctx, root, artifact).await?;
    }

    Ok(())
}

async fn fetch<D>(ctx: &AppContext, entry: &Locked<D>) -> Result<Vec<Artifact>> {
    let mut artifacts = Vec::new();

    for download in entry.package.downloads() {
        let key = download.run(ctx).await?;
        let path = ctx.store.object_path(&key);

        artifacts.push(Artifact {
            path: entry.directory.join(download.destination()),
            hash: key.to_hex(),
            size: size_of(&path).await?,
        });
    }

    Ok(artifacts)
}

async fn all_in_store(ctx: &AppContext, artifacts: &[Artifact]) -> Result<bool> {
    for artifact in artifacts {
        let key = ObjectKey::from_hex(&artifact.hash)?;

        if !ctx.store.object_exists(&key).await? {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn place_artifact(ctx: &AppContext, root: &Path, artifact: &Artifact) -> Result<()> {
    let key = ObjectKey::from_hex(&artifact.hash)?;
    let source = ctx.store.object_path(&key);
    let destination = root.join(&artifact.path);

    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("could not create `{}`", parent.display()))?;
    }

    if tokio::fs::symlink_metadata(&destination).await.is_ok() {
        tokio::fs::remove_file(&destination)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("could not replace `{}`", destination.display()))?;
    }

    if tokio::fs::hard_link(&source, &destination).await.is_ok() {
        return Ok(());
    }

    tokio::fs::copy(&source, &destination)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not write `{}`", destination.display()))?;

    Ok(())
}

async fn size_of(path: &Path) -> Result<u64> {
    let metadata = tokio::fs::metadata(path)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not measure `{}`", path.display()))?;

    Ok(metadata.len())
}
