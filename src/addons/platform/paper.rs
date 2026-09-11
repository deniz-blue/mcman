use crate::{addons::platform::PlatformType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperPlatform {
    pub minecraft: String,
}

impl PlatformType for PaperPlatform {
    const TYPE_NAME: &'static str = "paper";

    fn read(reader: &mut Reader) -> Self {
        Self {
            minecraft: reader.required_property("minecraft"),
        }
    }

    fn minecraft_version(&self) -> Option<&str> {
        Some(&self.minecraft)
    }
}
