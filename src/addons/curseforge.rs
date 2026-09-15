use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurseForgeAddon {
    pub project: String,
    pub version: Option<String>,
}

impl KdlVariant for CurseForgeAddon {
    const TYPE_NAME: &'static str = "curseforge";
}

impl KdlRead for CurseForgeAddon {
    fn read(reader: &mut Reader) -> Self {
        Self {
            project: reader.required_argument_or_property("project"),
            version: reader.property("version"),
        }
    }
}

impl KdlWrite for CurseForgeAddon {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.project.as_str());
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
    }
}
