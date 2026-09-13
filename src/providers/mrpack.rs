use std::io::Cursor;

use zip::ZipArchive;

use crate::{
    core::{
        checksum::{ChecksumAlgorithm, Checksums},
        location::Location,
        AppContext,
    },
    manifest::MrpackInclude,
    modpack::{
        mrpack::{Mrpack, MrpackError},
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
    let mut pack = Mrpack::open(archive)?;
    let downloads = pack.downloads(side).collect();
    let artifacts = pack.store_overrides(side, &ctx.store).await?;

    Ok(Resolved {
        resolved: MrpackInclude {
            location: include.location.clone(),
            checksums,
        },
        package: Package::from_downloads(downloads),
        artifacts,
    })
}
