use kdl::KdlNode;

use crate::{
    addons::platform::{minecraft::MinecraftPlatform, PlatformDependencies},
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForgePlatform {
    pub loader: Option<String>,
}

impl KdlVariant for ForgePlatform {
    const TYPE_NAME: &'static str = "forge";
}

impl PlatformDependencies for ForgePlatform {
    const REQUIRES: &'static [&'static str] = &[MinecraftPlatform::TYPE_NAME];
    const ACCEPTS: &'static [&'static str] = &["forge"];
}

impl KdlRead for ForgePlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            loader: reader.property("loader"),
        }
    }
}

impl KdlWrite for ForgePlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
    }
}
