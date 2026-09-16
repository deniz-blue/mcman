use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpongePlatform {
    pub api: Option<String>,
}

impl KdlVariant for SpongePlatform {
    const TYPE_NAME: &'static str = "sponge";
}

impl PlatformDependencies for SpongePlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
    const ACCEPTS: &'static [&'static str] = &["sponge"];
}

impl KdlRead for SpongePlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            api: reader.property("api"),
        }
    }
}

impl KdlWrite for SpongePlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(api) = &self.api {
            node.push(("api", api.as_str()));
        }
    }
}
