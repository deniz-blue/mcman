use std::path::Path;

use crate::store::{
    error::{io, StoreError},
    paths,
};

#[derive(Debug)]
pub struct StoreLock(std::fs::File);

impl StoreLock {
    pub(super) async fn shared(store: &Path) -> Result<Self, StoreError> {
        let path = paths::lock(store);
        tokio::task::spawn_blocking(move || {
            let file = open(&path)?;
            file.lock_shared().map_err(io("could not lock", &path))?;
            Ok(StoreLock(file))
        })
        .await
        .expect("locking should not panic")
    }

    pub(super) fn try_exclusive(store: &Path) -> Result<Self, StoreError> {
        let path = paths::lock(store);
        let file = open(&path)?;

        match file.try_lock() {
            Ok(()) => Ok(StoreLock(file)),
            Err(std::fs::TryLockError::WouldBlock) => Err(StoreError::InUse {
                path: store.to_owned(),
            }),
            Err(std::fs::TryLockError::Error(source)) => Err(io("could not lock", &path)(source)),
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        self.0.unlock().ok();
    }
}

fn open(path: &Path) -> Result<std::fs::File, StoreError> {
    std::fs::File::create(path).map_err(io("could not create", path))
}
