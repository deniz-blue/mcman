use std::fmt::Display;

use kdl::KdlNode;

use crate::{
    addons::platform::{
        fabric::FabricPlatform, minecraft::MinecraftPlatform, paper::PaperPlatform,
        velocity::VelocityPlatform,
    },
    core::kdl::{write_entries, Errors, Reader, Spanned},
};

pub mod fabric;
pub mod minecraft;
pub mod paper;
pub mod velocity;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Platform {
    Minecraft(MinecraftPlatform),
    Paper(PaperPlatform),
    Velocity(VelocityPlatform),
    Fabric(FabricPlatform),
}

pub trait PlatformType: Sized {
    const TYPE_NAME: &'static str;
    const REQUIRES: &'static [&'static str] = &[];

    fn read(reader: &mut Reader) -> Self;

    fn write(&self, node: &mut KdlNode);
}

fn type_names() -> String {
    [
        MinecraftPlatform::TYPE_NAME,
        PaperPlatform::TYPE_NAME,
        VelocityPlatform::TYPE_NAME,
        FabricPlatform::TYPE_NAME,
    ]
    .join(", ")
}

impl Platform {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Minecraft(_) => MinecraftPlatform::TYPE_NAME,
            Self::Paper(_) => PaperPlatform::TYPE_NAME,
            Self::Velocity(_) => VelocityPlatform::TYPE_NAME,
            Self::Fabric(_) => FabricPlatform::TYPE_NAME,
        }
    }

    pub fn requires(&self) -> &'static [&'static str] {
        match self {
            Self::Minecraft(_) => MinecraftPlatform::REQUIRES,
            Self::Paper(_) => PaperPlatform::REQUIRES,
            Self::Velocity(_) => VelocityPlatform::REQUIRES,
            Self::Fabric(_) => FabricPlatform::REQUIRES,
        }
    }

    pub fn write(&self, node: &mut KdlNode) {
        node.push(self.type_name());

        match self {
            Self::Minecraft(platform) => platform.write(node),
            Self::Paper(platform) => platform.write(node),
            Self::Velocity(platform) => platform.write(node),
            Self::Fabric(platform) => platform.write(node),
        }
    }

    pub fn read(node: &KdlNode, errors: &mut Errors) -> Option<Spanned<Self>> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let type_name = reader.required_argument("platform type");

        let platform = match type_name.as_str() {
            MinecraftPlatform::TYPE_NAME => Self::Minecraft(PlatformType::read(&mut reader)),
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

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut node = KdlNode::new("platform");
        self.write(&mut node);
        write_entries(&node, f)
    }
}
