use kdl::KdlNode;

use crate::core::kdl::{debug_without_span, Errors, Reader};

const PLATFORM_NAMES: &str = "paper, velocity, fabric";

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Platform {
    Paper(PaperPlatform),
    Velocity(VelocityPlatform),
    Fabric(FabricPlatform),
}

pub trait PlatformContext {
    fn name(&self) -> &'static str;

    fn minecraft_version(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PaperPlatform {
    pub minecraft: String,
    pub span: miette::SourceSpan,
}

#[derive(Clone, PartialEq, Eq)]
pub struct VelocityPlatform {
    pub span: miette::SourceSpan,
}

#[derive(Clone, PartialEq, Eq)]
pub struct FabricPlatform {
    pub minecraft: String,
    pub loader: Option<String>,
    pub span: miette::SourceSpan,
}

debug_without_span!(PaperPlatform { minecraft });
debug_without_span!(VelocityPlatform {});
debug_without_span!(FabricPlatform { minecraft, loader });

impl PlatformContext for PaperPlatform {
    fn name(&self) -> &'static str {
        "paper"
    }

    fn minecraft_version(&self) -> Option<&str> {
        Some(&self.minecraft)
    }
}

impl PlatformContext for VelocityPlatform {
    fn name(&self) -> &'static str {
        "velocity"
    }
}

impl PlatformContext for FabricPlatform {
    fn name(&self) -> &'static str {
        "fabric"
    }

    fn minecraft_version(&self) -> Option<&str> {
        Some(&self.minecraft)
    }
}

impl PlatformContext for Platform {
    fn name(&self) -> &'static str {
        self.context().name()
    }

    fn minecraft_version(&self) -> Option<&str> {
        self.context().minecraft_version()
    }
}

impl Platform {
    pub fn span(&self) -> miette::SourceSpan {
        match self {
            Self::Paper(paper) => paper.span,
            Self::Velocity(velocity) => velocity.span,
            Self::Fabric(fabric) => fabric.span,
        }
    }

    fn context(&self) -> &dyn PlatformContext {
        match self {
            Self::Paper(paper) => paper,
            Self::Velocity(velocity) => velocity,
            Self::Fabric(fabric) => fabric,
        }
    }

    pub(super) fn read(node: &KdlNode, errors: &mut Errors) -> Option<Self> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let name = reader.required_argument("platform name");

        let platform = match name.as_str() {
            "paper" => Some(Self::Paper(PaperPlatform {
                minecraft: reader.required_property("minecraft"),
                span,
            })),
            "velocity" => Some(Self::Velocity(VelocityPlatform { span })),
            "fabric" => Some(Self::Fabric(FabricPlatform {
                minecraft: reader.required_property("minecraft"),
                loader: reader.property("loader"),
                span,
            })),
            _ => None,
        };

        let Some(platform) = platform else {
            errors.push(
                span,
                format!("unknown platform `{name}`, expected one of: {PLATFORM_NAMES}"),
            );
            return None;
        };

        reader.reject_unread();
        Some(platform)
    }
}
