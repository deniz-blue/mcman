use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformType},
    core::kdl::Reader,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperPlatform {
    pub build: Option<String>,
}

impl PlatformType for PaperPlatform {
    const TYPE_NAME: &'static str = "paper";
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];

    fn read(reader: &mut Reader) -> Self {
        Self {
            build: reader.property("build"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
