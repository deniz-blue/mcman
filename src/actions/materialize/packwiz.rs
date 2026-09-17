use std::{collections::BTreeMap, path::Path};

use miette::{bail, IntoDiagnostic, Result, WrapErr};

use crate::{
    addons::Platform,
    core::{checksum::ChecksumAlgorithm, AppContext},
    lockfile::{Locked, LockedTarget},
    modpack::packwiz::{
        PackwizMetafile, PackwizMetafileDownload, PackwizPackHeader, PackwizPackWriter,
    },
    package::source::download::Download,
    store::ObjectKey,
};

pub async fn materialize(ctx: &AppContext, root: &Path, target: &LockedTarget) -> Result<()> {
    let mut writer = PackwizPackWriter::new(root);

    for addon in &target.addons {
        add_entry(ctx, &mut writer, addon).await?;
    }

    for include in &target.includes {
        add_entry(ctx, &mut writer, include).await?;
    }

    for package in &target.packages {
        for artifact in &package.artifacts {
            add_object(ctx, &mut writer, &artifact.path, &artifact.hash).await?;
        }
    }

    writer
        .finish(PackwizPackHeader {
            name: target.meta.get("name").unwrap_or(&target.name).clone(),
            author: target.meta.get("author").cloned(),
            version: target.meta.get("version").cloned(),
            description: target.meta.get("description").cloned(),
            versions: versions(target)?,
        })
        .await
}

async fn add_entry<D>(
    ctx: &AppContext,
    writer: &mut PackwizPackWriter<'_>,
    entry: &Locked<D>,
) -> Result<()> {
    let mut linked = Vec::new();

    for download in entry.package.downloads() {
        let destination = entry.directory.join(download.destination());

        writer
            .write_metafile(&destination, &metafile(ctx, download).await?)
            .await?;

        linked.push(destination);
    }

    for artifact in &entry.artifacts {
        if linked.contains(&artifact.path) {
            continue;
        }

        add_object(ctx, writer, &artifact.path, &artifact.hash).await?;
    }

    Ok(())
}

async fn add_object(
    ctx: &AppContext,
    writer: &mut PackwizPackWriter<'_>,
    path: &Path,
    hash: &str,
) -> Result<()> {
    let key = ObjectKey::from_hex(hash)?;

    writer.add_file(path, &ctx.store.object_path(&key)).await
}

async fn metafile(ctx: &AppContext, download: &Download) -> Result<PackwizMetafile> {
    let destination = download.destination();
    let name = |part: Option<&std::ffi::OsStr>| {
        part.expect("a download destination ends in a file name")
            .to_string_lossy()
            .into_owned()
    };

    let (hash_format, hash) = pinned(ctx, download).await?;

    Ok(PackwizMetafile {
        name: name(destination.file_stem()),
        filename: name(destination.file_name()),
        download: PackwizMetafileDownload {
            url: download.url.clone(),
            hash,
            hash_format,
        },
    })
}

async fn pinned(ctx: &AppContext, download: &Download) -> Result<(ChecksumAlgorithm, String)> {
    if let Some((algorithm, digest)) = download.checksums.strongest() {
        return Ok((algorithm, digest.to_owned()));
    }

    let key = download.run(ctx).await?;
    let path = ctx.store.object_path(&key);
    let bytes = tokio::fs::read(&path)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read `{}`", path.display()))?;

    let algorithm = ChecksumAlgorithm::Sha256;

    Ok((algorithm, algorithm.digest(&bytes)))
}

fn versions(target: &LockedTarget) -> Result<BTreeMap<String, String>> {
    let mut versions = BTreeMap::new();

    for platform in &target.platforms {
        let (name, version) = match &platform.resolved {
            Platform::Minecraft(minecraft) => ("minecraft", minecraft.version.clone()),
            Platform::Fabric(fabric) => ("fabric", fabric.loader.clone()),
            Platform::Quilt(quilt) => ("quilt", quilt.loader.clone()),
            Platform::NeoForge(neoforge) => ("neoforge", neoforge.loader.clone()),
            Platform::Forge(forge) => ("forge", forge.loader.clone()),
            Platform::Bukkit(_)
            | Platform::Spigot(_)
            | Platform::Paper(_)
            | Platform::Purpur(_)
            | Platform::Folia(_)
            | Platform::Sponge(_)
            | Platform::Velocity(_)
            | Platform::BungeeCord(_) => bail!(
                "a packwiz pack has no place for `{}`",
                platform.resolved.type_name()
            ),
        };

        let Some(version) = version else {
            bail!(
                "`{}` needs a version to go in a packwiz pack",
                platform.resolved.type_name()
            );
        };

        versions.insert(name.to_owned(), version);
    }

    Ok(versions)
}
