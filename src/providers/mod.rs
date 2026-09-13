use crate::{
    addons::{AddonType, Platform},
    lockfile::Artifact,
    package::Package,
};

pub mod download;
mod error;
pub mod modrinth;
pub mod mrpack;

pub use error::ProviderError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved<A> {
    pub resolved: A,
    pub package: Package,
    pub artifacts: Vec<Artifact>,
}

impl<A> Resolved<A> {
    pub fn new(resolved: A, package: Package) -> Self {
        Self {
            resolved,
            package,
            artifacts: Vec::new(),
        }
    }

    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> Resolved<B> {
        Resolved {
            resolved: f(self.resolved),
            package: self.package,
            artifacts: self.artifacts,
        }
    }
}

pub trait AddonResolver {
    type Addon: AddonType;

    async fn resolve(
        &self,
        addon: &Self::Addon,
        platforms: &[Platform],
    ) -> Result<Resolved<Self::Addon>, ProviderError>;
}
