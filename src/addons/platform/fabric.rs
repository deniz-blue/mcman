use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricPlatform {
    pub loader: Option<String>,
}

impl KdlVariant for FabricPlatform {
    const TYPE_NAME: &'static str = "fabric";
}

impl PlatformDependencies for FabricPlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
}

impl KdlRead for FabricPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            loader: reader.property("loader"),
        }
    }
}

impl KdlWrite for FabricPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
    }
}
