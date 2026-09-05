use std::path::PathBuf;

use knus::Decode;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

use crate::{core::AppContext, store::ObjectKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUrl {
    pub content_hash: ObjectKey,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub fetched_at: Option<u128>,
}

fn epoch_now() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct Download {
    #[knus(argument)]
    pub url: String,
    #[knus(property, str)]
    pub path: Option<PathBuf>,
}

impl Download {
    pub async fn run(&self, ctx: &AppContext) -> Result<ObjectKey> {
        let response = ctx
            .http
            .get(&self.url)
            .send()
            .await
            .into_diagnostic()?
            .error_for_status()
            .into_diagnostic()?;

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let mut stream = response.bytes_stream();

        let file_path = ctx.store.temp_file().await?;
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .into_diagnostic()?;
        let mut hasher = blake3::Hasher::new();

        while let Some(chunk) = stream.try_next().await.into_diagnostic()? {
            hasher.update(&chunk);
            tokio::io::copy(&mut chunk.as_ref(), &mut file)
                .await
                .into_diagnostic()?;
        }

        let key = ObjectKey::from(hasher.finalize());

        file.flush().await.into_diagnostic()?;
        drop(file);

        ctx.store.move_to_object_store(&file_path, &key).await?;
        tokio::fs::remove_file(&file_path).await.into_diagnostic()?;

        let _cached = CachedUrl {
            content_hash: key.clone(),
            etag,
            last_modified,
            fetched_at: Some(epoch_now()),
        };

        Ok(key)
    }
}
