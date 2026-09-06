use std::{path::PathBuf, str::FromStr};

use kdl::KdlNode;
use miette::{miette, Result};

use crate::core::kdl::{debug_without_span, Errors, Reader};

#[derive(Clone, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    pub path: Option<PathBuf>,
    pub kind: TargetType,
    pub span: miette::SourceSpan,
}

debug_without_span!(Target { name, path, kind });

impl Target {
    pub(super) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
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

        Self {
            name,
            path,
            kind,
            span,
        }
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
