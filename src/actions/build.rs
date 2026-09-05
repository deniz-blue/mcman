use std::path::Path;

use miette::{IntoDiagnostic, Result};

use crate::{
    core::AppContext,
    manifest::{Manifest, Preset},
    package::{source::PackageSource, Package},
    plan::{self, PlanWarning, TargetPlan},
};

pub async fn build_manifest_all(ctx: &AppContext, path: &Path, manifest: &Manifest) -> Result<()> {
    let plan = plan::from_manifest(manifest)?;

    for warning in &plan.warnings {
        match warning {
            PlanWarning::GroupWithoutTarget { label: Some(label) } => {
                eprintln!("warning: group `{label}` reaches no target")
            }
            PlanWarning::GroupWithoutTarget { label: None } => {
                eprintln!("warning: the manifest declares no targets")
            }
        }
    }

    for target in &plan.targets {
        build_manifest_target(ctx, path, target).await?;
    }

    Ok(())
}

pub async fn build_manifest_target(
    ctx: &AppContext,
    manifest_path: &Path,
    plan: &TargetPlan,
) -> Result<()> {
    let target_path = if let Some(path) = &plan.target.path {
        manifest_path.join(path)
    } else {
        manifest_path.to_path_buf()
    };

    for directory in &plan.directories {
        let dir_path = if let Some(path) = &directory.path {
            target_path.join(path)
        } else {
            target_path.to_path_buf()
        };

        for package in &directory.packages {
            build_package(ctx, &dir_path, package).await?;
        }

        for preset in &directory.presets {
            build_preset(ctx, &dir_path, preset).await?;
        }
    }

    Ok(())
}

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
    let build_dir = ctx.store.temp_dir().await?;

    for source in &package.sources {
        match source {
            PackageSource::Download(download) => {
                let key = download.run(ctx).await?;
                tokio::fs::copy(ctx.store.object_path(&key), &build_dir)
                    .await
                    .into_diagnostic()?;
            }
            PackageSource::Git(_) => todo!(),
        }
    }

    Ok(())
}

pub async fn build_preset(_ctx: &AppContext, _path: &Path, _preset: &Preset) -> Result<()> {
    Ok(())
}
