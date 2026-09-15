use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VelocityPlatform {
    pub version: Option<String>,
    pub build: Option<String>,
}

impl KdlVariant for VelocityPlatform {
    const TYPE_NAME: &'static str = "velocity";
}

impl KdlRead for VelocityPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            version: reader.property("version"),
            build: reader.property("build"),
        }
    }
}

impl KdlWrite for VelocityPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
