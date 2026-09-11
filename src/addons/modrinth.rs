use std::fmt::Display;

use crate::{
    addons::{AddonType, RequestedVersion},
    core::kdl::Reader,
    providers::modrinth::ModrinthProjectId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModrinthAddon {
    pub project: ModrinthProjectId,
    pub version: Option<RequestedVersion>,
}

impl AddonType for ModrinthAddon {
    const TYPE_NAME: &'static str = "modrinth";

    fn read(name: &str, reader: &mut Reader) -> Self {
        Self {
            project: ModrinthProjectId(name.to_owned()),
            version: reader.property("version").map(RequestedVersion::from),
        }
    }
}

impl Display for ModrinthAddon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", Self::TYPE_NAME, self.project)
    }
}
