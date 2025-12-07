use futures::TryStreamExt;
use miette::{IntoDiagnostic, Result};

use crate::{ctx::Context, task::TaskHandle};

use std::path::PathBuf;

use super::Task;

use std::path::Path;

pub struct DownloadTask<'a> {
    pub url: &'a str,
    pub destination: &'a Path,
}

impl<'a> Task for DownloadTask<'a> {
    type Output = PathBuf;

    async fn run(&self, ctx: &Context, _handle: &TaskHandle) -> Result<Self::Output> {
        let mut stream = ctx
            .http
            .get(self.url)
            .send()
            .await
            .into_diagnostic()?
            .error_for_status()
            .into_diagnostic()?
            .bytes_stream();

        let mut file = tokio::fs::File::create(self.destination)
            .await
            .into_diagnostic()?;

        while let Some(bytes) = stream.try_next().await.into_diagnostic()? {
            tokio::io::copy(&mut bytes.as_ref(), &mut file)
                .await
                .into_diagnostic()?;
        }

        Ok(self.destination.to_path_buf())
    }
}
