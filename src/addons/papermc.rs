use std::fmt::Display;

use crate::{
    addons::{AddonType, RequestedVersion},
    core::kdl::Reader,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperMcAddon {
    pub project: String,
    pub version: Option<RequestedVersion>,
}

impl AddonType for PaperMcAddon {
    const TYPE_NAME: &'static str = "papermc";

    fn read(name: &str, reader: &mut Reader) -> Self {
        Self {
            project: name.to_owned(),
            version: reader.property("version").map(RequestedVersion::from),
        }
    }
}

impl Display for PaperMcAddon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", Self::TYPE_NAME, self.project)
    }
}
