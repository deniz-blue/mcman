use miette::Result;

use crate::{core::AppContext, lockfile::Artifact, store::ObjectKey};

pub mod build;
pub mod materialize;

pub async fn all_in_store(ctx: &AppContext, artifacts: &[Artifact]) -> Result<bool> {
    for artifact in artifacts {
        let key = ObjectKey::from_hex(&artifact.hash)?;

        if !ctx.store.object_exists(&key).await? {
            return Ok(false);
        }
    }

    Ok(true)
}
