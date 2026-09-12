use kdl::KdlNode;

use crate::{addons::platform::PlatformType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperPlatform {
    pub minecraft: Option<String>,
    pub build: Option<String>,
}

impl PlatformType for PaperPlatform {
    const TYPE_NAME: &'static str = "paper";

    fn read(reader: &mut Reader) -> Self {
        Self {
            minecraft: reader.property("minecraft"),
            build: reader.property("build"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        if let Some(minecraft) = &self.minecraft {
            node.push(("minecraft", minecraft.as_str()));
        }
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }

    fn minecraft_version(&self) -> Option<&str> {
        self.minecraft.as_deref()
    }
}
