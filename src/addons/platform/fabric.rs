use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformType},
    core::kdl::Reader,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricPlatform {
    pub loader: Option<String>,
}

impl PlatformType for FabricPlatform {
    const TYPE_NAME: &'static str = "fabric";
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];

    fn read(reader: &mut Reader) -> Self {
        Self {
            loader: reader.property("loader"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
    }
}
