use mcman::core::checksum::{ChecksumAlgorithm, Checksums};

fn digest(algorithm: ChecksumAlgorithm, body: &[u8]) -> String {
    let mut hasher = algorithm.hasher();
    hasher.update(body);
    hex::encode(hasher.finalize())
}

#[test]
fn each_algorithm_hashes_what_its_name_says() {
    assert_eq!(
        digest(ChecksumAlgorithm::Sha1, b"abc"),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    assert_eq!(
        digest(ChecksumAlgorithm::Sha256, b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        digest(ChecksumAlgorithm::Sha512, b"abc"),
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
         2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
    );
}

#[test]
fn a_digest_is_as_long_as_its_algorithm_claims() {
    for algorithm in ChecksumAlgorithm::ALL {
        assert_eq!(digest(algorithm, b"abc").len(), algorithm.digest_length());
    }
}

#[test]
fn the_strongest_checksum_wins_whatever_order_they_arrived_in() {
    let mut checksums = Checksums::default();
    checksums.insert(ChecksumAlgorithm::Sha512, "512".to_owned());
    checksums.insert(ChecksumAlgorithm::Sha1, "1".to_owned());

    assert_eq!(
        checksums.strongest(),
        Some((ChecksumAlgorithm::Sha512, "512"))
    );
}

#[test]
fn a_lone_weak_checksum_is_still_the_strongest_one() {
    let mut checksums = Checksums::default();
    checksums.insert(ChecksumAlgorithm::Sha1, "1".to_owned());

    assert_eq!(checksums.strongest(), Some((ChecksumAlgorithm::Sha1, "1")));
}

#[test]
fn no_checksums_means_nothing_to_verify() {
    assert!(Checksums::default().is_empty());
    assert_eq!(Checksums::default().strongest(), None);
}
