use std::fmt::Display;

use kdl::KdlNode;

use crate::{
    addons::{
        curseforge::CurseForgeAddon, fabric::FabricLoaderAddon, github::GitHubAddon,
        hangar::HangarAddon, maven::MavenAddon, modrinth::ModrinthAddon, papermc::PaperMcAddon,
    },
    core::kdl::Reader,
};

pub mod curseforge;
pub mod fabric;
pub mod github;
pub mod hangar;
pub mod maven;
pub mod modrinth;
pub mod papermc;
pub mod platform;
pub mod version;

pub use platform::{Platform, PlatformType};
pub use version::{RequestedVersion, Stability};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Addon {
    Modrinth(ModrinthAddon),
    PaperMc(PaperMcAddon),
    FabricLoader(FabricLoaderAddon),
    Hangar(HangarAddon),
    CurseForge(CurseForgeAddon),
    GitHub(GitHubAddon),
    Maven(MavenAddon),
}

pub trait AddonType: Display + Sized {
    const TYPE_NAME: &'static str;

    fn read(reader: &mut Reader) -> Self;

    fn write(&self, node: &mut KdlNode);
}

fn type_names() -> String {
    [
        ModrinthAddon::TYPE_NAME,
        PaperMcAddon::TYPE_NAME,
        FabricLoaderAddon::TYPE_NAME,
        HangarAddon::TYPE_NAME,
        CurseForgeAddon::TYPE_NAME,
        GitHubAddon::TYPE_NAME,
        MavenAddon::TYPE_NAME,
    ]
    .join(", ")
}

impl Addon {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Modrinth(_) => ModrinthAddon::TYPE_NAME,
            Self::PaperMc(_) => PaperMcAddon::TYPE_NAME,
            Self::FabricLoader(_) => FabricLoaderAddon::TYPE_NAME,
            Self::Hangar(_) => HangarAddon::TYPE_NAME,
            Self::CurseForge(_) => CurseForgeAddon::TYPE_NAME,
            Self::GitHub(_) => GitHubAddon::TYPE_NAME,
            Self::Maven(_) => MavenAddon::TYPE_NAME,
        }
    }

    pub fn read(reader: &mut Reader) -> Option<Self> {
        let type_name = reader.required_argument("addon type");

        Some(match type_name.as_str() {
            ModrinthAddon::TYPE_NAME => Self::Modrinth(AddonType::read(reader)),
            PaperMcAddon::TYPE_NAME => Self::PaperMc(AddonType::read(reader)),
            FabricLoaderAddon::TYPE_NAME => Self::FabricLoader(AddonType::read(reader)),
            HangarAddon::TYPE_NAME => Self::Hangar(AddonType::read(reader)),
            CurseForgeAddon::TYPE_NAME => Self::CurseForge(AddonType::read(reader)),
            GitHubAddon::TYPE_NAME => Self::GitHub(AddonType::read(reader)),
            MavenAddon::TYPE_NAME => Self::Maven(AddonType::read(reader)),
            _ => {
                reader.reject(format!(
                    "unknown addon type `{type_name}`, expected one of: {}",
                    type_names()
                ));
                return None;
            }
        })
    }

    pub fn write(&self, node: &mut KdlNode) {
        node.push(self.type_name());

        match self {
            Self::Modrinth(addon) => addon.write(node),
            Self::PaperMc(addon) => addon.write(node),
            Self::FabricLoader(addon) => addon.write(node),
            Self::Hangar(addon) => addon.write(node),
            Self::CurseForge(addon) => addon.write(node),
            Self::GitHub(addon) => addon.write(node),
            Self::Maven(addon) => addon.write(node),
        }
    }

}

impl Display for Addon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modrinth(addon) => addon.fmt(f),
            Self::PaperMc(addon) => addon.fmt(f),
            Self::FabricLoader(addon) => addon.fmt(f),
            Self::Hangar(addon) => addon.fmt(f),
            Self::CurseForge(addon) => addon.fmt(f),
            Self::GitHub(addon) => addon.fmt(f),
            Self::Maven(addon) => addon.fmt(f),
        }
    }
}
