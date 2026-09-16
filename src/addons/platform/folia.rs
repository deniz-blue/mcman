use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoliaPlatform {
    pub build: Option<String>,
}

impl KdlVariant for FoliaPlatform {
    const TYPE_NAME: &'static str = "folia";
}

impl PlatformDependencies for FoliaPlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
    const ACCEPTS: &'static [&'static str] = &["folia"];
}

impl KdlRead for FoliaPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            build: reader.property("build"),
        }
    }
}

impl KdlWrite for FoliaPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
