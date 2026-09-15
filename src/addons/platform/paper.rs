use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperPlatform {
    pub build: Option<String>,
}

impl KdlVariant for PaperPlatform {
    const TYPE_NAME: &'static str = "paper";
}

impl PlatformDependencies for PaperPlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
}

impl KdlRead for PaperPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            build: reader.property("build"),
        }
    }
}

impl KdlWrite for PaperPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
