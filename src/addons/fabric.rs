use std::fmt::Display;

use crate::{
    addons::{AddonType, RequestedVersion},
    core::kdl::Reader,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricAddon {
    pub name: String,
    pub version: Option<RequestedVersion>,
}

impl AddonType for FabricAddon {
    const TYPE_NAME: &'static str = "fabric";

    fn read(name: &str, reader: &mut Reader) -> Self {
        Self {
            name: name.to_owned(),
            version: reader.property("version").map(RequestedVersion::from),
        }
    }
}

impl Display for FabricAddon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", Self::TYPE_NAME, self.name)
    }
}
