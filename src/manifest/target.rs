use std::{path::PathBuf, str::FromStr};

use kdl::KdlNode;
use miette::{miette, Result};

use crate::core::kdl::{Errors, Reader, Spanned};

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
        let kind = reader.property("type");
        reader.reject_unread();

        let kind = match kind {
            Some(kind) => match TargetType::from_str(&kind) {
                Ok(kind) => kind,
                Err(error) => {
                    errors.push(span, error.to_string());
                    TargetType::None
                }
            },
            None => TargetType::None,
        };

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
            _ => Err(miette!("Invalid target type: {}", s)),
        }
    }
}
