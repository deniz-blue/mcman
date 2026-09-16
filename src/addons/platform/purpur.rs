use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurpurPlatform {
    pub build: Option<String>,
}

impl KdlVariant for PurpurPlatform {
    const TYPE_NAME: &'static str = "purpur";
}

impl PlatformDependencies for PurpurPlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
    const ACCEPTS: &'static [&'static str] = &["purpur", "paper", "spigot", "bukkit"];
}

impl KdlRead for PurpurPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            build: reader.property("build"),
        }
    }
}

impl KdlWrite for PurpurPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
