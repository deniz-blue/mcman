use kdl::KdlNode;

use crate::{
    addons::platform::PlatformDependencies,
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BungeeCordPlatform {
    pub version: Option<String>,
    pub build: Option<String>,
}

impl KdlVariant for BungeeCordPlatform {
    const TYPE_NAME: &'static str = "bungeecord";
}

impl PlatformDependencies for BungeeCordPlatform {
    const REQUIRES: &'static [&'static str] = &[];
    const ACCEPTS: &'static [&'static str] = &["bungeecord"];
}

impl KdlRead for BungeeCordPlatform {
    fn read(reader: &mut Reader) -> Self {
        Self {
            version: reader.property("version"),
            build: reader.property("build"),
        }
    }
}

impl KdlWrite for BungeeCordPlatform {
    fn write(&self, node: &mut KdlNode) {
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
        if let Some(build) = &self.build {
            node.push(("build", build.as_str()));
        }
    }
}
