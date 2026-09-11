use crate::{addons::platform::PlatformType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricPlatform {
    pub minecraft: String,
    pub loader: Option<String>,
}

impl PlatformType for FabricPlatform {
    const TYPE_NAME: &'static str = "fabric";

    fn read(reader: &mut Reader) -> Self {
        Self {
            minecraft: reader.required_property("minecraft"),
            loader: reader.property("loader"),
        }
    }

    fn minecraft_version(&self) -> Option<&str> {
        Some(&self.minecraft)
    }
}
