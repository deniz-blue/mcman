use std::{path::Path, sync::Arc};

use mcman::{
    core::{
        checksum::{ChecksumAlgorithm, Checksums},
        location::Location,
        AppContext,
    },
    lockfile::LockedTarget,
    manifest::{Include, Manifest, MrpackInclude},
    package::source::PackageSource,
    plan,
    resolve::{self, ResolveError},
    store::Store,
};
use tempfile::TempDir;

mod common;

const PACK_ENTRIES: [(&str, &[u8]); 3] = [
    ("overrides/config/shared.toml", b"shared"),
    ("overrides/config/both.toml", b"generic"),
    ("server-overrides/config/both.toml", b"for servers"),
];

async fn resolve(checksum: Option<&str>) -> (TempDir, Result<LockedTarget, ResolveError>) {
    let root = TempDir::new().expect("a temp dir");
    let bytes = common::mrpack(&PACK_ENTRIES);
    std::fs::write(root.path().join("Pack-1.2.mrpack"), &bytes).expect("writes the pack");

    let declared = match checksum {
        Some(digest) => format!(" sha512=\"{digest}\""),
        None => String::new(),
    };
    let text = format!(
        "platform minecraft version=\"1.21.1\"\n\
         include mrpack \"Pack-1.2.mrpack\"{declared}\n\
         target \"smp\" path=\"./run/smp\" type=\"server\"\n"
    );

    let path = root.path().join("mcman.kdl");
    std::fs::write(&path, &text).expect("writes the manifest");

    let manifest = Manifest::parse(&path.to_string_lossy(), &text).expect("the manifest parses");
    let plan = plan::from_manifest(&manifest).expect("the manifest plans");

    let store = Store::open(root.path().join("store"))
        .await
        .expect("the store opens");
    let ctx = AppContext::new(Arc::new(store));

    let locked = resolve::resolve_target(&ctx, &Location::Path(path), &plan.targets[0], None).await;

    (root, locked)
}

#[tokio::test]
async fn a_pack_include_locks_the_files_its_side_keeps() {
    let (_root, locked) = resolve(None).await;
    let locked = locked.expect("the pack resolves");
    let include = &locked.includes[0];

    let downloads: Vec<_> = include
        .package
        .sources
        .iter()
        .map(|source| match source {
            PackageSource::Download(download) => download.destination().to_owned(),
            PackageSource::Git(_) => unreachable!("a pack only brings downloads"),
        })
        .collect();

    assert_eq!(
        downloads,
        [Path::new("mods/lithium.jar"), Path::new("mods/spark.jar")]
    );

    let overrides: Vec<_> = include
        .artifacts
        .iter()
        .map(|artifact| artifact.path.as_path())
        .collect();
    assert_eq!(
        overrides,
        [
            Path::new("config/both.toml"),
            Path::new("config/shared.toml")
        ]
    );
    assert_eq!(
        include.artifacts[0].hash,
        blake3::hash(b"for servers").to_hex().as_str()
    );
}

#[tokio::test]
async fn a_resolved_pack_pins_the_archive_it_read() {
    let (_root, locked) = resolve(None).await;
    let locked = locked.expect("the pack resolves");

    let Include::Mrpack(resolved) = &locked.includes[0].resolved else {
        unreachable!("an mrpack include resolves to an mrpack include");
    };

    let expected = ChecksumAlgorithm::Sha512.digest(&common::mrpack(&PACK_ENTRIES));
    assert_eq!(
        resolved.checksums.get(ChecksumAlgorithm::Sha512),
        Some(expected.as_str())
    );

    assert_eq!(
        locked.includes[0].requested,
        Include::Mrpack(MrpackInclude {
            location: Location::Path("Pack-1.2.mrpack".into()),
            checksums: Checksums::default(),
        })
    );
}

#[tokio::test]
async fn a_pack_that_does_not_match_its_checksum_is_rejected() {
    let (_root, locked) = resolve(Some(&"0".repeat(128))).await;
    let error = locked.expect_err("the checksum does not match");

    assert!(
        error.to_string().contains("Pack-1.2.mrpack"),
        "expected the pack in the error, got: {error}"
    );
    assert!(
        error.source.to_string().contains("expected"),
        "expected a mismatch, got: {}",
        error.source
    );
}
