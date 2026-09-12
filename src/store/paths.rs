use std::path::{Path, PathBuf};

use crate::store::ObjectKey;

pub(super) fn objects(store: &Path) -> PathBuf {
    store.join("objects")
}

pub(super) fn tmp(store: &Path) -> PathBuf {
    store.join("tmp")
}

pub(super) fn http(store: &Path) -> PathBuf {
    store.join("http")
}

pub(super) fn lock(store: &Path) -> PathBuf {
    store.join("lock")
}

pub(super) fn object(store: &Path, key: &ObjectKey) -> PathBuf {
    let hash = key.to_hex();
    objects(store).join(&hash[0..2]).join(hash)
}
