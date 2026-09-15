use std::{fmt::Display, path::PathBuf, str::FromStr};

use kdl::KdlNode;
use miette::{miette, Result};

use crate::{
    core::kdl::{KdlRead, KdlWrite, Reader},
    modpack::Side,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    pub path: Option<PathBuf>,
    pub kind: TargetType,
}

impl KdlRead for Target {
    fn read(reader: &mut Reader) -> Self {
        Self {
            name: reader.required_argument("target name"),
            path: reader.path_property("path"),
            kind: KdlRead::read(reader),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TargetType {
    #[default]
    Files,
    Client,
    Server,
    Packwiz,
    Mrpack,
    Unsup,
}

impl TargetType {
    pub const ALL: [Self; 6] = [
        Self::Files,
        Self::Client,
        Self::Server,
        Self::Packwiz,
        Self::Mrpack,
        Self::Unsup,
    ];

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Files => "files",
            Self::Client => "client",
            Self::Server => "server",
            Self::Packwiz => "packwiz",
            Self::Mrpack => "mrpack",
            Self::Unsup => "unsup",
        }
    }

    pub fn side(&self) -> Option<Side> {
        match self {
            Self::Server => Some(Side::Server),
            Self::Client => Some(Side::Client),
            _ => None,
        }
    }
}

impl KdlRead for TargetType {
    fn read(reader: &mut Reader) -> Self {
        let Some(text) = reader.property("type") else {
            return Self::Files;
        };

        match Self::from_str(&text) {
            Ok(kind) => kind,
            Err(error) => {
                reader.reject(error.to_string());
                Self::Files
            }
        }
    }
}

impl KdlWrite for TargetType {
    fn write(&self, node: &mut KdlNode) {
        if *self != Self::Files {
            node.push(("type", self.type_name()));
        }
    }
}

impl FromStr for TargetType {
    type Err = miette::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "files" | "" => Ok(TargetType::Files),
            "client" => Ok(TargetType::Client),
            "server" => Ok(TargetType::Server),
            "packwiz" => Ok(TargetType::Packwiz),
            "mrpack" => Ok(TargetType::Mrpack),
            "unsup" => Ok(TargetType::Unsup),
            _ => Err(miette!(
                "unknown target type `{s}`, expected one of: {}",
                type_names()
            )),
        }
    }
}

impl Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.type_name())
    }
}

fn type_names() -> String {
    TargetType::ALL
        .iter()
        .map(TargetType::type_name)
        .collect::<Vec<_>>()
        .join(", ")
}
