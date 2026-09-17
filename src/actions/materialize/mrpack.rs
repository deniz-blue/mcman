use std::{collections::BTreeMap, io::Cursor, path::Path};

use miette::{bail, IntoDiagnostic, Result, WrapErr};

use crate::{
    addons::Platform,
    core::{
        checksum::{ChecksumAlgorithm, Checksums},
        fs::create_parent,
        AppContext,
    },
    lockfile::{Locked, LockedTarget},
    modpack::mrpack::{MrpackFile, MrpackHeader, MrpackWriter},
    package::source::download::Download,
    store::ObjectKey,
};

// modrinth.index.json wants a sha1 and a sha512 for every file it lists.
const INDEX_HASHES: [ChecksumAlgorithm; 2] = [ChecksumAlgorithm::Sha1, ChecksumAlgorithm::Sha512];

pub async fn materialize(ctx: &AppContext, root: &Path, target: &LockedTarget) -> Result<()> {
    let mut writer = MrpackWriter::new(Cursor::new(Vec::new()));

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

    let Some(version) = target.meta.get("version") else {
        bail!("an mrpack needs a version, so declare `meta version=\"…\"`");
    };

    let archive = writer.finish(MrpackHeader {
        name: target.meta.get("name").unwrap_or(&target.name).clone(),
        version_id: version.clone(),
        summary: target.meta.get("description").cloned(),
        dependencies: dependencies(target)?,
    })?;

    let path = root.join(format!("{}.mrpack", target.name));
    create_parent(&path).await?;

    tokio::fs::write(&path, archive.into_inner())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not write `{}`", path.display()))
}

async fn add_entry<D>(
    ctx: &AppContext,
    writer: &mut MrpackWriter<Cursor<Vec<u8>>>,
    entry: &Locked<D>,
) -> Result<()> {
    let mut listed = Vec::new();

    for download in entry.package.downloads() {
        let destination = entry.directory.join(download.destination());
        writer.add_download(listed_file(ctx, &destination, download).await?);
        listed.push(destination);
    }

    for artifact in &entry.artifacts {
        if listed.contains(&artifact.path) {
            continue;
        }

        add_object(ctx, writer, &artifact.path, &artifact.hash).await?;
    }

    Ok(())
}

async fn add_object(
    ctx: &AppContext,
    writer: &mut MrpackWriter<Cursor<Vec<u8>>>,
    path: &Path,
    hash: &str,
) -> Result<()> {
    let key = ObjectKey::from_hex(hash)?;
    let source = ctx.store.object_path(&key);
    let bytes = tokio::fs::read(&source)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read `{}`", source.display()))?;

    Ok(writer.add_override(path, None, &bytes)?)
}

async fn listed_file(ctx: &AppContext, path: &Path, download: &Download) -> Result<MrpackFile> {
    let (hashes, file_size) = pinned(ctx, download).await?;

    Ok(MrpackFile {
        path: path.to_owned(),
        hashes,
        env: None,
        downloads: vec![download.url.clone()],
        file_size,
    })
}

async fn pinned(ctx: &AppContext, download: &Download) -> Result<(Checksums, u64)> {
    let complete = INDEX_HASHES
        .iter()
        .all(|algorithm| download.checksums.get(*algorithm).is_some());

    if let (true, Some(size)) = (complete, download.size) {
        return Ok((download.checksums.clone(), size));
    }

    let key = download.run(ctx).await?;
    let path = ctx.store.object_path(&key);
    let bytes = tokio::fs::read(&path)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read `{}`", path.display()))?;

    let mut hashes = download.checksums.clone();
    for algorithm in INDEX_HASHES {
        if hashes.get(algorithm).is_none() {
            hashes.insert(algorithm, algorithm.digest(&bytes));
        }
    }

    Ok((hashes, bytes.len() as u64))
}

fn dependencies(target: &LockedTarget) -> Result<BTreeMap<String, String>> {
    let mut dependencies = BTreeMap::new();

    for platform in &target.platforms {
        let (name, version) = match &platform.resolved {
            Platform::Minecraft(minecraft) => ("minecraft", minecraft.version.clone()),
            Platform::Fabric(fabric) => ("fabric-loader", fabric.loader.clone()),
            Platform::Quilt(quilt) => ("quilt-loader", quilt.loader.clone()),
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
                "an mrpack has no place for `{}`",
                platform.resolved.type_name()
            ),
        };

        let Some(version) = version else {
            bail!(
                "`{}` needs a version to go in an mrpack",
                platform.resolved.type_name()
            );
        };

        dependencies.insert(name.to_owned(), version);
    }

    Ok(dependencies)
}
