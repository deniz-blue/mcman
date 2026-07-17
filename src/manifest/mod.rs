use std::{path::PathBuf, str::FromStr};

use knus::Decode;
use miette::miette;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Manifest {
    pub root: Group,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Group {
    #[knus(children(name = "dir"))]
    pub directories: Vec<Directory>,
    #[knus(children(name = "target"))]
    pub targets: Vec<Target>,
    #[knus(children(name = "group"))]
    pub subgroups: Vec<Group>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Directory {
    #[knus(argument)]
    pub path: Option<PathBuf>,
    #[knus(children(name = "dependency"))]
    pub dependencies: Vec<Dependency>,
    #[knus(children(name = "package"))]
    pub packages: Vec<Package>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Target {
    #[knus(argument)]
    pub name: String,
    #[knus(property, str)]
    pub path: Option<PathBuf>,
    #[knus(property(name = "type"), str)]
    pub kind: TargetType,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum TargetType {
    #[default]
    None,
    Client,
    Server,
    Packwiz,
    Mrpack,
    Unsup,
}

impl FromStr for TargetType {
    type Err = miette::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(TargetType::Client),
            "server" => Ok(TargetType::Server),
            "packwiz" => Ok(TargetType::Packwiz),
            "mrpack" => Ok(TargetType::Mrpack),
            "unsup" => Ok(TargetType::Unsup),
            _ => Err(miette!("Invalid target type: {}", s)),
        }
    }
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub identifier: String,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Package {
    pub sources: Vec<PackageSource>,
    pub build: Option<PackageBuild>,
    pub links: Vec<PackageLink>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PackageSource {
    Download(Download),
    Git(Git),
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Download {
    pub url: String,
    pub path: Option<PathBuf>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Git {
    pub url: String,
    pub path: Option<PathBuf>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PackageBuild {
    pub tasks: Vec<BuildTask>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BuildTask {
    Execute(ExecuteTask),
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ExecuteTask {
    #[knus(argument)]
    pub command: String,
    #[knus(property(name = "cd"))]
    pub directory: Option<PathBuf>,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PackageLink {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: Option<PathBuf>,
}
