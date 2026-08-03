use std::path::PathBuf;

use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Lockfile {
    #[knus(child)]
    pub meta: LockfileMeta,
    #[knus(children(name = "target"))]
    pub targets: Vec<LockedTarget>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct LockfileMeta {
    #[knus(property)]
    pub version: u8,
    #[knus(property)]
    pub generated: String,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
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
pub struct LockedPreset {
    #[knus(argument)]
    pub identifier: String,
    #[knus(property)]
    pub version: String,
    #[knus(children(name = "artifact"))]
    pub artifacts: Vec<Artifact>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct LockedPackage {
    #[knus(argument)]
    pub name: String,
    #[knus(children(name = "artifact"))]
    pub artifacts: Vec<Artifact>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Artifact {
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
}
