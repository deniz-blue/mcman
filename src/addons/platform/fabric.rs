use kdl::KdlNode;

use crate::{addons::platform::PlatformType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricPlatform {
    pub minecraft: Option<String>,
    pub loader: Option<String>,
}

impl PlatformType for FabricPlatform {
    const TYPE_NAME: &'static str = "fabric";

    fn read(reader: &mut Reader) -> Self {
        Self {
            minecraft: reader.property("minecraft"),
            loader: reader.property("loader"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        if let Some(minecraft) = &self.minecraft {
            node.push(("minecraft", minecraft.as_str()));
        }
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
    }

    fn minecraft_version(&self) -> Option<&str> {
        self.minecraft.as_deref()
    }
}
