use std::{collections::BTreeMap, path::Path};

use miette::{bail, Context, IntoDiagnostic, Result};

use crate::{
    addons::Platform,
    core::{checksum::ChecksumAlgorithm, AppContext},
    lockfile::{Artifact, Locked, LockedTarget},
    modpack::packwiz::{
        slashed, Metafile, MetafileDownload, Pack, PackFile, PackIndex, INDEX_TOML, PACK_FORMAT,
        PACK_TOML,
    },
    package::source::download::Download,
    store::ObjectKey,
};

const INDEX_HASHES: ChecksumAlgorithm = ChecksumAlgorithm::Sha256;

pub async fn materialize(ctx: &AppContext, root: &Path, target: &LockedTarget) -> Result<()> {
    let mut files = Vec::new();

    for addon in &target.addons {
        write_entry(ctx, root, addon, &mut files).await?;
    }

    for include in &target.includes {
        write_entry(ctx, root, include, &mut files).await?;
    }

    for package in &target.packages {
        for artifact in &package.artifacts {
            files.push(write_file(ctx, root, artifact).await?);
        }
    }

    files.sort_by(|left, right| left.file.cmp(&right.file));

    let index = PackIndex {
        hash_format: INDEX_HASHES,
        files,
    };
    let index = write_toml(root, Path::new(INDEX_TOML), &index).await?;

    let pack = Pack {
        name: target.name.clone(),
        pack_format: PACK_FORMAT.to_owned(),
        index,
        versions: versions(target)?,
    };

    write_toml(root, Path::new(PACK_TOML), &pack).await?;

    Ok(())
}

async fn write_entry<D>(
    ctx: &AppContext,
    root: &Path,
    entry: &Locked<D>,
    files: &mut Vec<PackFile>,
) -> Result<()> {
    let mut linked = Vec::new();

    for download in entry.package.downloads() {
        let destination = entry.directory.join(download.destination());
        linked.push(destination.clone());
        files.push(write_metafile(ctx, root, &destination, download).await?);
    }

    for artifact in &entry.artifacts {
        if linked.contains(&artifact.path) {
            continue;
        }

        files.push(write_file(ctx, root, artifact).await?);
    }

    Ok(())
}

async fn write_metafile(
    ctx: &AppContext,
    root: &Path,
    destination: &Path,
    download: &Download,
) -> Result<PackFile> {
    let name = |part: Option<&std::ffi::OsStr>| {
        part.expect("a download destination ends in a file name")
            .to_string_lossy()
            .into_owned()
    };

    let (hash_format, hash) = pinned(ctx, download).await?;

    let entry = Metafile {
        name: name(destination.file_stem()),
        filename: name(destination.file_name()),
        download: MetafileDownload {
            url: download.url.clone(),
            hash,
            hash_format,
        },
    };

    let mut path = destination.to_owned();
    path.set_extension("pw.toml");

    let mut file = write_toml(root, &path, &entry).await?;
    file.metafile = true;

    Ok(file)
}

async fn pinned(ctx: &AppContext, download: &Download) -> Result<(ChecksumAlgorithm, String)> {
    if let Some((algorithm, digest)) = download.checksums.strongest() {
        return Ok((algorithm, digest.to_owned()));
    }

    let key = download.run(ctx).await?;
    let bytes = read_object(ctx, &key).await?;

    Ok((INDEX_HASHES, INDEX_HASHES.digest(&bytes)))
}

async fn write_file(ctx: &AppContext, root: &Path, artifact: &Artifact) -> Result<PackFile> {
    let key = ObjectKey::from_hex(&artifact.hash)?;
    let bytes = read_object(ctx, &key).await?;

    write_bytes(root, &artifact.path, &bytes).await?;

    Ok(pack_file(&artifact.path, INDEX_HASHES.digest(&bytes)))
}

async fn write_toml(root: &Path, path: &Path, value: &impl serde::Serialize) -> Result<PackFile> {
    let text = toml::to_string_pretty(value)
        .into_diagnostic()
        .wrap_err_with(|| format!("could not write `{}`", path.display()))?;

    write_bytes(root, path, text.as_bytes()).await?;

    Ok(pack_file(path, INDEX_HASHES.digest(text.as_bytes())))
}

fn pack_file(path: &Path, hash: String) -> PackFile {
    PackFile {
        file: slashed(path),
        hash,
        metafile: false,
    }
}

async fn read_object(ctx: &AppContext, key: &ObjectKey) -> Result<Vec<u8>> {
    let path = ctx.store.object_path(key);

    tokio::fs::read(&path)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read `{}`", path.display()))
}

async fn write_bytes(root: &Path, path: &Path, bytes: &[u8]) -> Result<()> {
    let destination = root.join(path);

    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("could not create `{}`", parent.display()))?;
    }

    tokio::fs::write(&destination, bytes)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("could not write `{}`", destination.display()))
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
