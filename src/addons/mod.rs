use std::fmt::Display;

use kdl::KdlNode;

use crate::core::kdl::{
    kdl_variants, write_entries, Errors, KdlMaybeRead, KdlRead, KdlVariant, KdlWrite, Reader,
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

pub use platform::Platform;

kdl_variants! {
    Addon reads "addon type" writes "use" {
        Modrinth => modrinth::ModrinthAddon,
        PaperMc => papermc::PaperMcAddon,
        FabricLoader => fabric::FabricLoaderAddon,
        Hangar => hangar::HangarAddon,
        CurseForge => curseforge::CurseForgeAddon,
        GitHub => github::GitHubAddon,
        Maven => maven::MavenAddon,
        Download => download::DownloadAddon,
    }
}
