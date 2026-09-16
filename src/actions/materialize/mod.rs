use std::path::Path;

use miette::{bail, Result};

use crate::{core::AppContext, lockfile::LockedTarget, manifest::TargetType};

mod files;
mod packwiz;

pub async fn materialize(ctx: &AppContext, root: &Path, target: &mut LockedTarget) -> Result<()> {
    match target.kind {
        TargetType::Files | TargetType::Client | TargetType::Server => {
            files::materialize(ctx, root, target).await
        }
        TargetType::Packwiz => packwiz::materialize(ctx, root, target).await,
        TargetType::Mrpack | TargetType::Unsup => {
            bail!("a `{}` target cannot be built yet", target.kind)
        }
    }
}
