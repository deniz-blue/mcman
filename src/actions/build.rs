use std::{collections::HashMap, path::Path};

use miette::{IntoDiagnostic, Result};

use crate::{
    core::AppContext,
    manifest::{Directory, Group, Manifest, Preset, Target},
    package::{source::PackageSource, Package},
};

pub async fn build_manifest_all(ctx: &AppContext, path: &Path, manifest: &Manifest) -> Result<()> {
    let mut todo = HashMap::<Target, Vec<Directory>>::new();

    fn process_group(todo: &mut HashMap<Target, Vec<Directory>>, group: &Group) {
        for target in &group.targets {
            todo.entry(target.clone())
                .or_insert_with(Vec::new)
                .extend(group.directories.clone());
        }

        for subgroup in &group.subgroups {
            process_group(todo, subgroup);
        }
    }

    process_group(&mut todo, &manifest.root);

    for (target, directories) in todo {
        build_manifest_target(ctx, path, &target, directories).await?;
    }

    Ok(())
}

pub async fn build_manifest_target(
    ctx: &AppContext,
    manifest_path: &Path,
    target: &Target,
    directories: Vec<Directory>,
) -> Result<()> {
    let target_path = if let Some(path) = &target.path {
        manifest_path.join(path)
    } else {
        manifest_path.to_path_buf()
    };

    for directory in directories {
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
                    let key = download.run(ctx).await?;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

pub async fn build_package_complex(
    ctx: &AppContext,
    output_path: &Path,
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

pub async fn build_preset(ctx: &AppContext, _path: &Path, _preset: &Preset) -> Result<()> {
    Ok(())
}
