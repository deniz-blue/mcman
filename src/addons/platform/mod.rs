use std::fmt::Display;

use kdl::KdlNode;

use crate::core::kdl::{
    kdl_variants, write_entries, Errors, KdlMaybeRead, KdlRead, KdlVariant, KdlWrite, Reader,
};

pub mod bukkit;
pub mod bungeecord;
pub mod fabric;
pub mod folia;
pub mod forge;
pub mod minecraft;
pub mod neoforge;
pub mod paper;
pub mod purpur;
pub mod quilt;
pub mod spigot;
pub mod sponge;
pub mod velocity;

pub trait PlatformDependencies {
    const REQUIRES: &'static [&'static str];
    const ACCEPTS: &'static [&'static str];
}

kdl_variants! {
    Platform reads "platform type" writes "platform" {
        Minecraft => minecraft::MinecraftPlatform,
        Bukkit => bukkit::BukkitPlatform,
        Spigot => spigot::SpigotPlatform,
        Paper => paper::PaperPlatform,
        Purpur => purpur::PurpurPlatform,
        Folia => folia::FoliaPlatform,
        Sponge => sponge::SpongePlatform,
        Velocity => velocity::VelocityPlatform,
        BungeeCord => bungeecord::BungeeCordPlatform,
        Fabric => fabric::FabricPlatform,
        Quilt => quilt::QuiltPlatform,
        NeoForge => neoforge::NeoForgePlatform,
        Forge => forge::ForgePlatform,
    }
}

macro_rules! platform_constant {
    ($platform:ident, $constant:ident) => {
        match $platform {
            Self::Minecraft(_) => minecraft::MinecraftPlatform::$constant,
            Self::Bukkit(_) => bukkit::BukkitPlatform::$constant,
            Self::Spigot(_) => spigot::SpigotPlatform::$constant,
            Self::Paper(_) => paper::PaperPlatform::$constant,
            Self::Purpur(_) => purpur::PurpurPlatform::$constant,
            Self::Folia(_) => folia::FoliaPlatform::$constant,
            Self::Sponge(_) => sponge::SpongePlatform::$constant,
            Self::Velocity(_) => velocity::VelocityPlatform::$constant,
            Self::BungeeCord(_) => bungeecord::BungeeCordPlatform::$constant,
            Self::Fabric(_) => fabric::FabricPlatform::$constant,
            Self::Quilt(_) => quilt::QuiltPlatform::$constant,
            Self::NeoForge(_) => neoforge::NeoForgePlatform::$constant,
            Self::Forge(_) => forge::ForgePlatform::$constant,
        }
    };
}

impl Platform {
    pub fn requires(&self) -> &'static [&'static str] {
        platform_constant!(self, REQUIRES)
    }

    pub fn accepts(&self) -> &'static [&'static str] {
        platform_constant!(self, ACCEPTS)
    }
}
