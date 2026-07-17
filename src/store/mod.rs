use std::path::PathBuf;

pub struct ObjectStoreKey(pub blake3::Hash);

pub struct ObjectStore {
    pub path: PathBuf,
}

impl ObjectStore {
    pub fn path(&self, key: &ObjectStoreKey) -> PathBuf {
        let hash = key.0.to_hex().to_string();
        self.path.join(hash[0..2].to_string()).join(hash)
    }

    pub async fn exists(&self, key: &ObjectStoreKey) -> Result<bool, std::io::Error> {
        let path = self.path(key);
        tokio::fs::try_exists(path).await
    }
}
