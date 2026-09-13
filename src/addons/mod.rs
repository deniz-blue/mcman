use std::fmt::Display;

use kdl::KdlNode;

use crate::{
    addons::{
        curseforge::CurseForgeAddon, download::DownloadAddon, fabric::FabricLoaderAddon,
        github::GitHubAddon, hangar::HangarAddon, maven::MavenAddon, modrinth::ModrinthAddon,
        papermc::PaperMcAddon,
    },
    core::kdl::{write_entries, Declaration, Errors, Reader, Spanned},
};

pub mod curseforge;
pub mod download;
pub mod fabric;
pub mod github;
pub mod hangar;
pub mod maven;
pub mod modrinth;
pub mod papermc;
pub mod platform;

pub use platform::{Platform, PlatformType};

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
    Download(DownloadAddon),
}

pub trait AddonType: Sized {
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
        DownloadAddon::TYPE_NAME,
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
            Self::Download(_) => DownloadAddon::TYPE_NAME,
        }
    }

    pub fn read_from(reader: &mut Reader) -> Option<Self> {
        let type_name = reader.required_argument("addon type");

        Some(match type_name.as_str() {
            ModrinthAddon::TYPE_NAME => Self::Modrinth(AddonType::read(reader)),
            PaperMcAddon::TYPE_NAME => Self::PaperMc(AddonType::read(reader)),
            FabricLoaderAddon::TYPE_NAME => Self::FabricLoader(AddonType::read(reader)),
            HangarAddon::TYPE_NAME => Self::Hangar(AddonType::read(reader)),
            CurseForgeAddon::TYPE_NAME => Self::CurseForge(AddonType::read(reader)),
            GitHubAddon::TYPE_NAME => Self::GitHub(AddonType::read(reader)),
            MavenAddon::TYPE_NAME => Self::Maven(AddonType::read(reader)),
            DownloadAddon::TYPE_NAME => Self::Download(AddonType::read(reader)),
            _ => {
                reader.reject(format!(
                    "unknown addon type `{type_name}`, expected one of: {}",
                    type_names()
                ));
                return None;
            }
        })
    }
}

impl Declaration for Addon {
    fn read(node: &KdlNode, errors: &mut Errors) -> Option<Spanned<Self>> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let addon = Self::read_from(&mut reader)?;
        reader.reject_unread();

        Some(Spanned::new(addon, span))
    }

    fn write(&self, node: &mut KdlNode) {
        node.push(self.type_name());

        match self {
            Self::Modrinth(addon) => addon.write(node),
            Self::PaperMc(addon) => addon.write(node),
            Self::FabricLoader(addon) => addon.write(node),
            Self::Hangar(addon) => addon.write(node),
            Self::CurseForge(addon) => addon.write(node),
            Self::GitHub(addon) => addon.write(node),
            Self::Maven(addon) => addon.write(node),
            Self::Download(addon) => addon.write(node),
        }
    }
}

impl From<ModrinthAddon> for Addon {
    fn from(addon: ModrinthAddon) -> Self {
        Self::Modrinth(addon)
    }
}

impl From<PaperMcAddon> for Addon {
    fn from(addon: PaperMcAddon) -> Self {
        Self::PaperMc(addon)
    }
}

impl From<FabricLoaderAddon> for Addon {
    fn from(addon: FabricLoaderAddon) -> Self {
        Self::FabricLoader(addon)
    }
}

impl From<HangarAddon> for Addon {
    fn from(addon: HangarAddon) -> Self {
        Self::Hangar(addon)
    }
}

impl From<CurseForgeAddon> for Addon {
    fn from(addon: CurseForgeAddon) -> Self {
        Self::CurseForge(addon)
    }
}

impl From<GitHubAddon> for Addon {
    fn from(addon: GitHubAddon) -> Self {
        Self::GitHub(addon)
    }
}

impl From<MavenAddon> for Addon {
    fn from(addon: MavenAddon) -> Self {
        Self::Maven(addon)
    }
}

impl From<DownloadAddon> for Addon {
    fn from(addon: DownloadAddon) -> Self {
        Self::Download(addon)
    }
}

impl Display for Addon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut node = KdlNode::new("use");
        self.write(&mut node);
        write_entries(&node, f)
    }
}
