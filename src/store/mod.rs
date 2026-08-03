use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result};

pub struct ObjectKey(pub blake3::Hash);

impl From<blake3::Hash> for ObjectKey {
    fn from(hash: blake3::Hash) -> Self {
        ObjectKey(hash)
    }
}

pub struct Store {
    pub path: PathBuf,
}

impl Store {
    pub fn object_path(&self, key: &ObjectKey) -> PathBuf {
        let hash = key.0.to_hex().to_string();
        self.path
            .join("objects")
            .join(hash[0..2].to_string())
            .join(hash)
    }

    pub async fn object_exists(&self, key: &ObjectKey) -> Result<bool> {
        let path = self.object_path(key);
        tokio::fs::try_exists(path).await.into_diagnostic()
    }

    pub async fn move_to_object_store(&self, from: &Path, key: &ObjectKey) -> Result<()> {
        let to = self.object_path(key);
        tokio::fs::create_dir_all(to.parent().unwrap())
            .await
            .into_diagnostic()?;
        tokio::fs::rename(from, to).await.into_diagnostic()?;
        Ok(())
    }

    pub async fn temp_file(&self) -> Result<PathBuf> {
        // NOTE: Needs rework!

        let temp_dir = self.path.join("temp");
        tokio::fs::create_dir_all(&temp_dir)
            .await
            .into_diagnostic()?;
        let temp_file = temp_dir.join(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        );
        tokio::fs::File::create(&temp_file)
            .await
            .into_diagnostic()?;
        Ok(temp_file)
    }
}
