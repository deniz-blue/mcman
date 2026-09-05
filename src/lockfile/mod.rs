use std::path::PathBuf;

use knus::Decode;
use miette::Result;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct Lockfile {
    #[knus(child)]
    pub meta: LockfileMeta,
    #[knus(children(name = "target"))]
    pub targets: Vec<LockedTarget>,
}

impl Lockfile {
    pub fn parse(name: &str, text: &str) -> Result<Self> {
        knus::parse(name, text).map_err(Into::into)
    }
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct LockfileMeta {
    #[knus(property)]
    pub version: u8,
    #[knus(property)]
    pub generated: String,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct LockedTarget {
    #[knus(argument)]
    pub name: String,
    #[knus(property)]
    pub path: PathBuf,
    #[knus(children(name = "use"))]
    pub presets: Vec<LockedPreset>,
    #[knus(children(name = "package"))]
    pub packages: Vec<LockedPackage>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct LockedPreset {
    #[knus(argument)]
    pub identifier: String,
    #[knus(property)]
    pub version: String,
    #[knus(children(name = "artifact"))]
    pub artifacts: Vec<Artifact>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct LockedPackage {
    #[knus(argument)]
    pub name: String,
    #[knus(children(name = "artifact"))]
    pub artifacts: Vec<Artifact>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct Artifact {
    #[knus(argument)]
    pub path: PathBuf,
    #[knus(property)]
    pub hash: String,
    #[knus(property)]
    pub size: u64,
}
