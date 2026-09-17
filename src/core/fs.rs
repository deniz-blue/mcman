use std::path::Path;

use miette::{IntoDiagnostic, Result, WrapErr};

pub async fn link_or_copy(source: &Path, destination: &Path) -> Result<()> {
    create_parent(destination).await?;

    if tokio::fs::symlink_metadata(destination).await.is_ok() {
        tokio::fs::remove_file(destination)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("could not replace `{}`", destination.display()))?;
    }

    if tokio::fs::hard_link(source, destination).await.is_ok() {
        return Ok(());
    }

    tokio::fs::copy(source, destination)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not write `{}`", destination.display()))?;

    Ok(())
}

pub async fn create_parent(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    tokio::fs::create_dir_all(parent)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not create `{}`", parent.display()))
}
