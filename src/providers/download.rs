use crate::{
    addons::{download::DownloadAddon, Platform},
    package::Package,
    providers::{AddonResolver, ProviderError, Resolved},
};

pub struct Downloads;

impl AddonResolver for Downloads {
    type Addon = DownloadAddon;

    async fn resolve(
        &self,
        addon: &DownloadAddon,
        _platform: Option<&Platform>,
    ) -> Result<Resolved<DownloadAddon>, ProviderError> {
        Ok(Resolved {
            resolved: addon.clone(),
            package: Package::from_downloads(vec![addon.0.clone()]),
        })
    }
}
