use std::path::Path;

use miette::{IntoDiagnostic, Result};
use tokio::io::AsyncReadExt;

use crate::{ctx::Context, hash::Hashes, task::{Task, TaskHandle}};

pub struct ChecksumTask<'a> {
    pub path: &'a Path,
    pub hashes: Hashes,
}

impl<'a> Task for ChecksumTask<'a> {
    type Output = ();

    // TODO: progress reporting
    async fn run(&self, _: &Context, _handle: &TaskHandle) -> Result<Self::Output> {
        let Some((hash_format, expected_hash)) = self.hashes.get_best_hash() else {
            return Ok(())
        };
        let mut hasher: Box<dyn digest::DynDigest> = (*hash_format).into();
        let file = tokio::fs::File::open(&self.path).await.into_diagnostic()?;
        let mut reader = tokio::io::BufReader::new(file);
        let mut buffer = [0u8; 8192];

        loop {
            let n = reader.read(&mut buffer).await.into_diagnostic()?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize_reset();
        let computed_hash = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        
        if &computed_hash != expected_hash {
            return Err(miette::miette!(
                "Checksum mismatch for {:?}: expected {}, got {}",
                self.path,
                expected_hash,
                computed_hash
            ));
        }

        Ok(())
    }
}
