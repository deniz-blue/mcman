use crate::{
    addons::{AddonType, Platform},
    package::Package,
};

mod error;
pub mod modrinth;

pub use error::ProviderError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved<A> {
    pub resolved: A,
    pub package: Package,
}

impl<A> Resolved<A> {
    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> Resolved<B> {
        Resolved {
            resolved: f(self.resolved),
            package: self.package,
        }
    }
}

pub trait AddonResolver {
    type Addon: AddonType;

    async fn resolve(
        &self,
        addon: &Self::Addon,
        platform: Option<&Platform>,
    ) -> Result<Resolved<Self::Addon>, ProviderError>;
}
