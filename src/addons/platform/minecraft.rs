use kdl::KdlNode;

use crate::{
    addons::platform::{Platform, PlatformType},
    core::kdl::Reader,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinecraftPlatform {
    pub version: Option<String>,
}

impl MinecraftPlatform {
    pub fn declared_in(platforms: &[Platform]) -> Option<&Self> {
        platforms.iter().find_map(|platform| match platform {
            Platform::Minecraft(minecraft) => Some(minecraft),
            _ => None,
        })
    }
}

impl PlatformType for MinecraftPlatform {
    const TYPE_NAME: &'static str = "minecraft";

    fn read(reader: &mut Reader) -> Self {
        Self {
            version: reader.property("version"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
    }
}
