use std::{io::Cursor, path::Path};

use mcman::{
    core::checksum::ChecksumAlgorithm,
    modpack::{mrpack::MrpackReader, Side},
};
use zip::ZipArchive;

mod common;

fn pack(entries: &[(&str, &[u8])]) -> MrpackReader<Cursor<Vec<u8>>> {
    let archive =
        ZipArchive::new(Cursor::new(common::mrpack(entries))).expect("the zip reads back");
    MrpackReader::open(archive).expect("the pack opens")
}

#[test]
fn a_server_target_drops_client_only_files() {
    let pack = pack(&[]);

    let paths: Vec<_> = pack
        .downloads(Some(Side::Server))
        .map(|download| download.destination().to_owned())
        .collect();

    assert_eq!(
        paths,
        [Path::new("mods/lithium.jar"), Path::new("mods/spark.jar")]
    );
    assert_eq!(pack.downloads(None).count(), 3);
    assert_eq!(pack.index.dependencies["minecraft"], "1.21.1");
}

#[test]
fn a_file_becomes_a_download_with_its_hashes_and_size() {
    let pack = pack(&[]);

    let sodium = pack
        .downloads(Some(Side::Client))
        .find(|download| download.destination() == Path::new("mods/sodium.jar"))
        .expect("a client keeps sodium");

    assert_eq!(
        sodium.url,
        "https://cdn.modrinth.com/data/AANobbMI/versions/def/sodium-0.5.jar"
    );
    assert_eq!(sodium.size, Some(2048));
    assert_eq!(
        sodium.checksums.get(ChecksumAlgorithm::Sha1),
        Some("fc8bb9919fa77c670b7ed5c284bcb66770ea71fd")
    );
    assert!(sodium.checksums.get(ChecksumAlgorithm::Sha512).is_some());
}

#[test]
fn side_overrides_win_over_shared_ones() {
    let mut pack = pack(&[
        ("overrides/config/shared.toml", b"shared"),
        ("overrides/config/both.toml", b"generic"),
        ("server-overrides/config/both.toml", b"for servers"),
        ("client-overrides/config/client.toml", b"for clients"),
    ]);

    let overrides: Vec<_> = pack
        .overrides(Some(Side::Server))
        .expect("the overrides resolve")
        .map(|entry| entry.expect("the entry reads"))
        .collect();

    let paths: Vec<_> = overrides.iter().map(|entry| &entry.path).collect();
    assert_eq!(
        paths,
        [
            Path::new("config/both.toml"),
            Path::new("config/shared.toml")
        ]
    );

    assert_eq!(overrides[0].bytes, b"for servers");
}
