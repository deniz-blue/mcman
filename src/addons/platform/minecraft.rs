use kdl::KdlNode;

use crate::{
    addons::platform::Platform,
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
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

impl KdlVariant for MinecraftPlatform {
    const TYPE_NAME: &'static str = "minecraft";
}

impl KdlRead for MinecraftPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            version: reader.property("version"),
        }
    }
}

impl KdlWrite for MinecraftPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
    }
}
