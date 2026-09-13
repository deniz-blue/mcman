use crate::{
    addons::{Addon, Platform},
    core::{kdl::Spanned, location::Location, AppContext},
    lockfile::{Locked, LockedPlatform, LockedTarget},
    manifest::Include,
    modpack::Side,
    plan::TargetPlan,
    providers::{
        download::Downloads, modrinth::Modrinth, mrpack, AddonResolver, ProviderError, Resolved,
    },
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

    result.map_err(|source| ResolveError::of(&addon.value, addon.span, source))
}

pub async fn resolve_include(
    ctx: &AppContext,
    manifest: &Location,
    include: &Spanned<Include>,
    side: Option<Side>,
) -> Result<Resolved<Include>, ResolveError> {
    let result = match &include.value {
        Include::Mrpack(inner) => mrpack::resolve(ctx, manifest, inner, side)
            .await
            .map(|resolved| resolved.map(Include::Mrpack)),
        other => Err(ProviderError::Unsupported {
            type_name: other.type_name(),
        }),
    };

    result.map_err(|source| ResolveError::of(&include.value, include.span, source))
}

pub async fn resolve_target(
    ctx: &AppContext,
    manifest: &Location,
    target: &TargetPlan,
    locked: Option<&LockedTarget>,
) -> Result<LockedTarget, ResolveError> {
    let platforms: Vec<Platform> = target
        .platforms
        .iter()
        .map(|platform| platform.value.clone())
        .collect();
    let side = target.target.kind.side();

    let mut runtimes = Vec::new();
    for runtime in &target.runtimes {
        let held = locked.and_then(|locked| find_locked(&locked.runtimes, &runtime.value));
        runtimes.push(match held {
            Some(held) => held.clone(),
            None => lock(
                &runtime.value,
                resolve_addon(ctx, runtime, &platforms).await?,
            ),
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
            None => lock(&addon.value, resolve_addon(ctx, addon, &platforms).await?),
        });
    }

    let mut includes = Vec::new();
    for include in &target.includes {
        let held = locked.and_then(|locked| find_locked(&locked.includes, &include.value));
        includes.push(match held {
            Some(held) => held.clone(),
            None => lock(
                &include.value,
                resolve_include(ctx, manifest, include, side).await?,
            ),
        });
    }

    let packages = locked
        .map(|locked| locked.packages.clone())
        .unwrap_or_default();

    Ok(LockedTarget {
        name: target.target.name.clone(),
        path: target.target.path.clone().unwrap_or_else(|| ".".into()),
        kind: target.target.kind.clone(),
        platforms: platforms
            .into_iter()
            .map(|platform| LockedPlatform {
                requested: platform.clone(),
                resolved: platform,
            })
            .collect(),
        runtimes,
        addons,
        includes,
        packages,
    })
}

fn find_locked<'a, D: PartialEq>(entries: &'a [Locked<D>], requested: &D) -> Option<&'a Locked<D>> {
    entries.iter().find(|entry| &entry.requested == requested)
}

fn lock<D: Clone>(requested: &D, resolved: Resolved<D>) -> Locked<D> {
    Locked {
        requested: requested.clone(),
        resolved: resolved.resolved,
        package: resolved.package,
        artifacts: resolved.artifacts,
    }
}
