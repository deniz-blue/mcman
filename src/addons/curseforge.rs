use std::fmt::Display;

use kdl::KdlNode;

use crate::{addons::AddonType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurseForgeAddon {
    pub project: String,
    pub version: Option<String>,
}

impl AddonType for CurseForgeAddon {
    const TYPE_NAME: &'static str = "curseforge";


    fn read(reader: &mut Reader) -> Self {
        Self {
            project: reader.required_argument_or_property("project"),
            version: reader.property("version"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        node.push(self.project.as_str());
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
    }
}

impl Display for CurseForgeAddon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", Self::TYPE_NAME, self.project)
    }
}
