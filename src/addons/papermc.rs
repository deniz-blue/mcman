use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperMcAddon {
    pub project: String,
    pub version: Option<String>,
    pub build: Option<String>,
}

impl KdlVariant for PaperMcAddon {
    const TYPE_NAME: &'static str = "papermc";
}

impl KdlRead for PaperMcAddon {
    fn read(reader: &mut Reader) -> Self {
        Self {
            project: reader.required_argument_or_property("project"),
            version: reader.property("version"),
            build: reader.property("build"),
        }
    }
}

impl KdlWrite for PaperMcAddon {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.project.as_str());
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
