use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuiltPlatform {
    pub loader: Option<String>,
}

impl KdlVariant for QuiltPlatform {
    const TYPE_NAME: &'static str = "quilt";
}

impl PlatformDependencies for QuiltPlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
    const ACCEPTS: &'static [&'static str] = &["quilt", "fabric"];
}

impl KdlRead for QuiltPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            loader: reader.property("loader"),
        }
    }
}

impl KdlWrite for QuiltPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
    }
}
