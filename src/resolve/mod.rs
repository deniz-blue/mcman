use crate::{
    addons::{Addon, Platform},
    core::{kdl::Spanned, AppContext},
    lockfile::{LockedAddon, LockedPlatform, LockedTarget},
    plan::TargetPlan,
    providers::{download::Downloads, modrinth::Modrinth, AddonResolver, ProviderError, Resolved},
};

mod error;

pub use error::ResolveError;

pub async fn resolve_addon(
    ctx: &AppContext,
    addon: &Spanned<Addon>,
    platforms: &[Platform],
) -> Result<Resolved<Addon>, ResolveError> {
    let result = match &addon.value {
        Addon::Modrinth(inner) => Modrinth::new(ctx.cached_http.clone())
            .resolve(inner, platforms)
            .await
            .map(|resolved| resolved.map(Addon::Modrinth)),
        Addon::Download(inner) => Downloads
            .resolve(inner, platforms)
            .await
            .map(|resolved| resolved.map(Addon::Download)),
        other => Err(ProviderError::Unsupported {
            type_name: other.type_name(),
        }),
    };

    result.map_err(|source| ResolveError {
        addon: addon.value.to_string(),
        at: addon.span,
        source,
    })
}

pub async fn resolve_target(
    ctx: &AppContext,
    target: &TargetPlan,
    locked: Option<&LockedTarget>,
) -> Result<LockedTarget, ResolveError> {
    let platforms: Vec<Platform> = target
        .platforms
        .iter()
        .map(|platform| platform.value.clone())
        .collect();

    if let Some(include) = target.includes.first() {
        return Err(ResolveError {
            addon: include.value.to_string(),
            at: include.span,
            source: ProviderError::Unsupported {
                type_name: "include",
            },
        });
    }

    let mut runtimes = Vec::new();
    for runtime in &target.runtimes {
        let held = locked.and_then(|locked| find_locked(&locked.runtimes, &runtime.value));
        runtimes.push(match held {
            Some(held) => held.clone(),
            None => lock_addon(ctx, runtime, &platforms).await?,
        });
    }

    let mut addons = Vec::new();
    for addon in target
        .directories
        .iter()
        .flat_map(|directory| &directory.addons)
    {
        let held = locked.and_then(|locked| find_locked(&locked.addons, &addon.value));
        addons.push(match held {
            Some(held) => held.clone(),
            None => lock_addon(ctx, addon, &platforms).await?,
        });
    }

    let packages = locked
        .map(|locked| locked.packages.clone())
        .unwrap_or_default();

    Ok(LockedTarget {
        name: target.target.name.clone(),
        path: target.target.path.clone().unwrap_or_else(|| ".".into()),
        platforms: platforms
            .into_iter()
            .map(|platform| LockedPlatform {
                requested: platform.clone(),
                resolved: platform,
            })
            .collect(),
        runtimes,
        addons,
        packages,
    })
}

fn find_locked<'a>(entries: &'a [LockedAddon], requested: &Addon) -> Option<&'a LockedAddon> {
    entries.iter().find(|entry| &entry.requested == requested)
}

async fn lock_addon(
    ctx: &AppContext,
    addon: &Spanned<Addon>,
    platforms: &[Platform],
) -> Result<LockedAddon, ResolveError> {
    let resolved = resolve_addon(ctx, addon, platforms).await?;

    Ok(LockedAddon {
        requested: addon.value.clone(),
        resolved: resolved.resolved,
        package: resolved.package,
        artifacts: Vec::new(),
    })
}
