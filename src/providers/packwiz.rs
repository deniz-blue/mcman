use std::path::PathBuf;

use crate::{
    core::{
        checksum::{ChecksumAlgorithm, Checksums},
        location::Location,
        AppContext,
    },
    lockfile::Artifact,
    manifest::PackwizInclude,
    modpack::packwiz::{PackwizContent, PackwizPackReader},
    package::Package,
    providers::{ProviderError, Resolved},
};

pub async fn resolve(
    ctx: &AppContext,
    manifest: &Location,
    include: &PackwizInclude,
) -> Result<Resolved<PackwizInclude>, ProviderError> {
    let location = manifest.join(&include.location.to_string())?;
    let reader = PackwizPackReader::open(ctx, location).await?;
    include.checksums.verify(reader.bytes())?;

    let mut checksums = Checksums::default();
    checksums.insert(
        ChecksumAlgorithm::Sha512,
        ChecksumAlgorithm::Sha512.digest(reader.bytes()),
    );

    let index = reader.fetch_index(ctx).await?;

    let mut downloads = Vec::new();
    let mut artifacts = Vec::new();

    for entry in index.entries() {
        match entry.read(ctx).await? {
            PackwizContent::Metafile(metafile) => {
                downloads.push(metafile.to_download(&entry.path()));
            }
            PackwizContent::File(bytes) => {
                artifacts.push(store(ctx, entry.path(), bytes).await?);
            }
        }
    }

    Ok(Resolved {
        resolved: PackwizInclude {
            location: include.location.clone(),
            checksums,
        },
        package: Package::from_downloads(downloads),
        artifacts,
    })
}

async fn store(ctx: &AppContext, path: PathBuf, bytes: Vec<u8>) -> Result<Artifact, ProviderError> {
    let mut object = ctx.store.write_object().await?;
    object.write(&bytes).await?;

    Ok(Artifact {
        path,
        hash: object.finish().await?.to_hex(),
        size: bytes.len() as u64,
    })
}
