use crate::{addons::platform::PlatformType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VelocityPlatform;

impl PlatformType for VelocityPlatform {
    const TYPE_NAME: &'static str = "velocity";

    fn read(_reader: &mut Reader) -> Self {
        Self
    }
}
