use std::{fmt::Display, path::PathBuf, str::FromStr};

use kdl::KdlNode;
use miette::{miette, Result};

use crate::{
    core::kdl::{Errors, Reader, Spanned},
    modpack::Side,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    pub path: Option<PathBuf>,
    pub kind: TargetType,
}

impl Target {
    pub(super) fn read(node: &KdlNode, errors: &mut Errors) -> Spanned<Self> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let name = reader.required_argument("target name");
        let path = reader.path_property("path");
        let kind = TargetType::read(&mut reader);
        reader.reject_unread();

        Spanned::new(Self { name, path, kind }, span)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum TargetType {
    #[default]
    None,
    Client,
    Server,
    Packwiz,
    Mrpack,
    Unsup,
}

impl TargetType {
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::Client,
        Self::Server,
        Self::Packwiz,
        Self::Mrpack,
        Self::Unsup,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Client => "client",
            Self::Server => "server",
            Self::Packwiz => "packwiz",
            Self::Mrpack => "mrpack",
            Self::Unsup => "unsup",
        }
    }

    pub fn read(reader: &mut Reader) -> Self {
        let Some(text) = reader.property("type") else {
            return Self::None;
        };

        match Self::from_str(&text) {
            Ok(kind) => kind,
            Err(error) => {
                reader.reject(error.to_string());
                Self::None
            }
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

impl FromStr for TargetType {
    type Err = miette::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" | "" => Ok(TargetType::None),
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
        f.write_str(self.name())
    }
}

fn type_names() -> String {
    TargetType::ALL
        .iter()
        .map(TargetType::name)
        .collect::<Vec<_>>()
        .join(", ")
}
