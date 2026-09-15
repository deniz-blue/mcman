use std::path::Path;

use miette::{IntoDiagnostic, Result};

use crate::{
    core::AppContext,
    package::{source::PackageSource, Package},
};

pub async fn build_package(ctx: &AppContext, output_path: &Path, package: &Package) -> Result<()> {
    if package
        .sources
        .iter()
        .any(|source| matches!(source, PackageSource::Git(_)))
    {
        build_package_complex(ctx, output_path, package).await?;
    } else {
        for source in &package.sources {
            match source {
                PackageSource::Download(download) => {
                    let _key = download.run(ctx).await?;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

pub async fn build_package_complex(
    ctx: &AppContext,
    _output_path: &Path,
    package: &Package,
) -> Result<()> {
    let build_dir = ctx.store.tmp()?;

    for source in &package.sources {
        match source {
            PackageSource::Download(download) => {
                let key = download.run(ctx).await?;
                let destination = build_dir.path().join(download.destination());

                if let Some(parent) = destination.parent() {
                    tokio::fs::create_dir_all(parent).await.into_diagnostic()?;
                }

                tokio::fs::copy(ctx.store.object_path(&key), destination)
                    .await
                    .into_diagnostic()?;
            }
            PackageSource::Git(_) => todo!(),
        }
    }

    Ok(())
}
