use std::path::{Path, PathBuf};

use tempfile::{NamedTempFile, TempDir};
use tokio::io::AsyncWriteExt;

mod error;
mod key;
mod lock;
mod paths;

pub use error::StoreError;
pub use key::ObjectKey;

use crate::store::{error::io, lock::StoreLock};

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    _held: StoreLock,
}

impl Store {
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let path = path.into();

        match Self::open_exclusive(&path).await {
            Ok(cleared) => drop(cleared),
            Err(StoreError::InUse { .. }) => {}
            Err(error) => return Err(error),
        }

        Ok(Self {
            _held: StoreLock::shared(&path).await?,
            path,
        })
    }

    pub async fn open_exclusive(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let path = path.into();
        create_directories(&path).await?;

        let store = Self {
            _held: StoreLock::try_exclusive(&path)?,
            path,
        };
        store.clear_tmp().await?;

        Ok(store)
    }

    pub fn http_cache_path(&self) -> PathBuf {
        paths::http(&self.path)
    }

    pub fn object_path(&self, key: &ObjectKey) -> PathBuf {
        paths::object(&self.path, key)
    }

    pub async fn object_exists(&self, key: &ObjectKey) -> Result<bool, StoreError> {
        let path = self.object_path(key);
        tokio::fs::try_exists(&path)
            .await
            .map_err(io("could not read", path))
    }

    pub async fn write_object(&self) -> Result<ObjectWriter<'_>, StoreError> {
        let handle = self.tmp_file()?;
        let file = handle
            .reopen()
            .map_err(io("could not open", handle.path()))?;

        Ok(ObjectWriter {
            store: self,
            file: tokio::fs::File::from_std(file),
            handle,
            hasher: blake3::Hasher::new(),
        })
    }

    pub async fn put(&self, file: &Path) -> Result<ObjectKey, StoreError> {
        let key = ObjectKey::of_file(file).await?;
        self.adopt(file, &key).await?;

        Ok(key)
    }

    pub fn tmp(&self) -> Result<TempDir, StoreError> {
        let root = paths::tmp(&self.path);
        TempDir::new_in(&root).map_err(io("could not create a directory in", root))
    }

    fn tmp_file(&self) -> Result<NamedTempFile, StoreError> {
        let root = paths::tmp(&self.path);
        NamedTempFile::new_in(&root).map_err(io("could not create a file in", root))
    }

    async fn clear_tmp(&self) -> Result<(), StoreError> {
        let root = paths::tmp(&self.path);
        let mut entries = tokio::fs::read_dir(&root)
            .await
            .map_err(io("could not read", &root))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(io("could not read", &root))?
        {
            let path = entry.path();
            let removed = if entry
                .file_type()
                .await
                .map_err(io("could not read", &path))?
                .is_dir()
            {
                tokio::fs::remove_dir_all(&path).await
            } else {
                tokio::fs::remove_file(&path).await
            };
            removed.map_err(io("could not remove", path))?;
        }

        Ok(())
    }

    async fn adopt(&self, file: &Path, key: &ObjectKey) -> Result<(), StoreError> {
        if self.object_exists(key).await? {
            return Ok(());
        }

        let destination = self.object_path(key);
        let parent = destination.parent().expect("an object path has a parent");
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(io("could not create", parent))?;

        read_only(file).await?;

        if tokio::fs::rename(file, &destination).await.is_ok() {
            return Ok(());
        }

        self.copy_across_filesystems(file, &destination).await
    }

    async fn copy_across_filesystems(
        &self,
        file: &Path,
        destination: &Path,
    ) -> Result<(), StoreError> {
        let staged = self.tmp_file()?;
        tokio::fs::copy(file, staged.path())
            .await
            .map_err(io("could not copy", file))?;
        read_only(staged.path()).await?;

        tokio::fs::rename(staged.path(), destination)
            .await
            .map_err(io("could not move into", destination))?;
        staged.into_temp_path().keep().ok();

        tokio::fs::remove_file(file)
            .await
            .map_err(io("could not remove", file))
    }
}

#[derive(Debug)]
pub struct ObjectWriter<'a> {
    store: &'a Store,
    handle: NamedTempFile,
    file: tokio::fs::File,
    hasher: blake3::Hasher,
}

impl ObjectWriter<'_> {
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), StoreError> {
        self.hasher.update(bytes);
        self.file
            .write_all(bytes)
            .await
            .map_err(io("could not write to", self.handle.path()))
    }

    pub async fn finish(mut self) -> Result<ObjectKey, StoreError> {
        self.file
            .flush()
            .await
            .map_err(io("could not write to", self.handle.path()))?;
        drop(self.file);

        let key = ObjectKey(self.hasher.finalize());
        let staged = self.handle.into_temp_path();
        self.store.adopt(&staged, &key).await?;
        staged.keep().ok();

        Ok(key)
    }
}

async fn create_directories(store: &Path) -> Result<(), StoreError> {
    match tokio::fs::metadata(store).await {
        Ok(found) if !found.is_dir() => {
            return Err(StoreError::NotADirectory {
                path: store.to_owned(),
            })
        }
        Ok(_) => {}
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => return Err(io("could not read", store)(source)),
    }

    for directory in [paths::objects(store), paths::tmp(store)] {
        tokio::fs::create_dir_all(&directory)
            .await
            .map_err(io("could not create", &directory))?;
    }

    Ok(())
}

async fn read_only(path: &Path) -> Result<(), StoreError> {
    let mut permissions = tokio::fs::metadata(path)
        .await
        .map_err(io("could not read", path))?
        .permissions();
    permissions.set_readonly(true);

    tokio::fs::set_permissions(path, permissions)
        .await
        .map_err(io("could not set permissions on", path))
}
