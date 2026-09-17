use std::path::Path;

use miette::{Context, IntoDiagnostic, Result};

use crate::{
    actions::all_in_store,
    core::{fs::link_or_copy, AppContext},
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

async fn place_artifact(ctx: &AppContext, root: &Path, artifact: &Artifact) -> Result<()> {
    let key = ObjectKey::from_hex(&artifact.hash)?;

    link_or_copy(&ctx.store.object_path(&key), &root.join(&artifact.path)).await
}

async fn size_of(path: &Path) -> Result<u64> {
    let metadata = tokio::fs::metadata(path)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not measure `{}`", path.display()))?;

    Ok(metadata.len())
}
