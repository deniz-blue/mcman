use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeoForgePlatform {
    pub loader: Option<String>,
}

impl KdlVariant for NeoForgePlatform {
    const TYPE_NAME: &'static str = "neoforge";
}

impl PlatformDependencies for NeoForgePlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
    const ACCEPTS: &'static [&'static str] = &["neoforge"];
}

impl KdlRead for NeoForgePlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            loader: reader.property("loader"),
        }
    }
}

impl KdlWrite for NeoForgePlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
    }
}
