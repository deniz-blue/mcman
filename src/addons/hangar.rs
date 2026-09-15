use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HangarAddon {
    pub project: String,
    pub version: Option<String>,
}

impl KdlVariant for HangarAddon {
    const TYPE_NAME: &'static str = "hangar";
}

impl KdlRead for HangarAddon {
    fn read(reader: &mut Reader) -> Self {
        Self {
            project: reader.required_argument_or_property("project"),
            version: reader.property("version"),
        }
    }
}

impl KdlWrite for HangarAddon {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.project.as_str());
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
    }
}
