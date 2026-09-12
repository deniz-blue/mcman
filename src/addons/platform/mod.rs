use kdl::KdlNode;

use crate::{
    addons::platform::{fabric::FabricPlatform, paper::PaperPlatform, velocity::VelocityPlatform},
    core::kdl::{Errors, Reader, Spanned},
};

pub mod fabric;
pub mod paper;
pub mod velocity;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Platform {
    Paper(PaperPlatform),
    Velocity(VelocityPlatform),
    Fabric(FabricPlatform),
}

pub trait PlatformType: Sized {
    const TYPE_NAME: &'static str;

    fn read(reader: &mut Reader) -> Self;

    fn minecraft_version(&self) -> Option<&str> {
        None
    }
}

fn type_names() -> String {
    [
        PaperPlatform::TYPE_NAME,
        VelocityPlatform::TYPE_NAME,
        FabricPlatform::TYPE_NAME,
    ]
    .join(", ")
}

impl Platform {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Paper(_) => PaperPlatform::TYPE_NAME,
            Self::Velocity(_) => VelocityPlatform::TYPE_NAME,
            Self::Fabric(_) => FabricPlatform::TYPE_NAME,
        }
    }

    pub fn minecraft_version(&self) -> Option<&str> {
        match self {
            Self::Paper(platform) => platform.minecraft_version(),
            Self::Velocity(platform) => platform.minecraft_version(),
            Self::Fabric(platform) => platform.minecraft_version(),
        }
    }

    pub fn read(node: &KdlNode, errors: &mut Errors) -> Option<Spanned<Self>> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let type_name = reader.required_argument("platform type");

        let platform = match type_name.as_str() {
            PaperPlatform::TYPE_NAME => Self::Paper(PlatformType::read(&mut reader)),
            VelocityPlatform::TYPE_NAME => Self::Velocity(PlatformType::read(&mut reader)),
            FabricPlatform::TYPE_NAME => Self::Fabric(PlatformType::read(&mut reader)),
            _ => {
                errors.push(
                    span,
                    format!(
                        "unknown platform `{type_name}`, expected one of: {}",
                        type_names()
                    ),
                );
                return None;
            }
        };

        reader.reject_unread();
        Some(Spanned::new(platform, span))
    }
}
