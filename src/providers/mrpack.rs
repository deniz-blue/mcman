use std::io::Cursor;

use zip::ZipArchive;

use crate::{
    core::{
        checksum::{ChecksumAlgorithm, Checksums},
        location::Location,
        AppContext,
    },
    lockfile::Artifact,
    manifest::MrpackInclude,
    modpack::{
        mrpack::{MrpackError, MrpackOverride, MrpackReader},
        Side,
    },
    package::Package,
    providers::{ProviderError, Resolved},
};

pub async fn resolve(
    ctx: &AppContext,
    manifest: &Location,
    include: &MrpackInclude,
    side: Option<Side>,
) -> Result<Resolved<MrpackInclude>, ProviderError> {
    let location = manifest.join(&include.location.to_string())?;
    let bytes = location.read_bytes(ctx).await?;
    include.checksums.verify(&bytes)?;

    let mut checksums = Checksums::default();
    checksums.insert(
        ChecksumAlgorithm::Sha512,
        ChecksumAlgorithm::Sha512.digest(&bytes),
    );

    let archive = ZipArchive::new(Cursor::new(bytes.as_slice())).map_err(MrpackError::Zip)?;
    let mut pack = MrpackReader::open(archive)?;
    let downloads = pack.downloads(side).collect();

    let mut artifacts = Vec::new();
    for entry in pack.overrides(side)? {
        artifacts.push(store(ctx, entry?).await?);
    }

    Ok(Resolved {
        resolved: MrpackInclude {
            location: include.location.clone(),
            checksums,
        },
        package: Package::from_downloads(downloads),
        artifacts,
    })
}

async fn store(ctx: &AppContext, entry: MrpackOverride) -> Result<Artifact, ProviderError> {
    let mut object = ctx.store.write_object().await?;
    object.write(&entry.bytes).await?;

    Ok(Artifact {
        path: entry.path,
        hash: object.finish().await?.to_hex(),
        size: entry.bytes.len() as u64,
    })
}
