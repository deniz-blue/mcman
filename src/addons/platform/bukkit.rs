use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BukkitPlatform {
    pub build: Option<String>,
}

impl KdlVariant for BukkitPlatform {
    const TYPE_NAME: &'static str = "bukkit";
}

impl PlatformDependencies for BukkitPlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
    const ACCEPTS: &'static [&'static str] = &["bukkit"];
}

impl KdlRead for BukkitPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            build: reader.property("build"),
        }
    }
}

impl KdlWrite for BukkitPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
