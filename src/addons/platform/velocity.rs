use kdl::KdlNode;

use crate::{addons::platform::PlatformType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VelocityPlatform {
    pub version: Option<String>,
    pub build: Option<String>,
}

impl PlatformType for VelocityPlatform {
    const TYPE_NAME: &'static str = "velocity";

    fn read(reader: &mut Reader) -> Self {
        Self {
            version: reader.property("version"),
            build: reader.property("build"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
