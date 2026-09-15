use std::path::{Path, PathBuf};

use kdl::KdlNode;
use miette::{bail, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::{
    core::{
        checksum::Checksums,
        kdl::{KdlRead, KdlWrite, Reader},
        AppContext,
    },
    store::ObjectKey,
};

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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Download {
    pub url: String,
    pub path: Option<PathBuf>,
    pub checksums: Checksums,
    pub size: Option<u64>,
}

impl KdlRead for Download {
    fn read(reader: &mut Reader) -> Self {
        let url = reader.required_argument("url");
        let path = reader.path_property("path");
        let checksums = KdlRead::read(reader);
        let size = reader.unsigned_property("size");

        if path.is_none() && file_name_in_url(&url).is_none() {
            reader.reject("`download` needs a `path` when its url has no file name".to_owned());
        }

        Self {
            url,
            path,
            checksums,
            size,
        }
    }
}

impl KdlWrite for Download {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.url.as_str());
        if let Some(path) = &self.path {
            node.push(("path", path.display().to_string()));
        }
        self.checksums.write(node);
        if let Some(size) = self.size {
            node.push(("size", i128::from(size)));
        }
    }
}

impl Download {
    pub fn destination(&self) -> &Path {
        match &self.path {
            Some(path) => path,
            None => Path::new(
                file_name_in_url(&self.url)
                    .expect("a url with no file name is rejected when the download parses"),
            ),
        }
    }

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

        let mut object = ctx.store.write_object().await?;
        let declared = self.checksums.strongest();
        let mut declared_hasher = declared.map(|(algorithm, _)| algorithm.hasher());
        let mut downloaded = 0u64;

        while let Some(chunk) = stream.try_next().await.into_diagnostic()? {
            if let Some(declared_hasher) = &mut declared_hasher {
                declared_hasher.update(&chunk);
            }
            downloaded += chunk.len() as u64;
            object.write(&chunk).await?;
        }

        if let Some(expected) = self.size {
            if downloaded != expected {
                bail!("{} is {downloaded} bytes, expected {expected}", self.url);
            }
        }

        if let (Some((algorithm, expected)), Some(declared_hasher)) = (declared, declared_hasher) {
            let found = hex::encode(declared_hasher.finalize());
            if !found.eq_ignore_ascii_case(expected) {
                bail!(
                    "{} has {} {found}, expected {expected}",
                    self.url,
                    algorithm.name()
                );
            }
        }

        let key = object.finish().await?;

        let _cached = CachedUrl {
            content_hash: key.clone(),
            etag,
            last_modified,
            fetched_at: Some(epoch_now()),
        };

        Ok(key)
    }
}

fn file_name_in_url(url: &str) -> Option<&str> {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let name = path.rsplit('/').next().unwrap_or_default();
    (!name.is_empty()).then_some(name)
}
