use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use kdl::KdlNode;

use crate::core::{
    checksum::Checksums,
    kdl::{write_entries, Errors, Reader, Spanned},
};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Include {
    Mrpack(MrpackInclude),
    Packwiz(PackwizInclude),
}

pub trait IncludeType: Sized {
    const TYPE_NAME: &'static str;

    fn read(reader: &mut Reader) -> Self;

    fn write(&self, node: &mut KdlNode);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MrpackInclude {
    pub location: Location,
    pub checksums: Checksums,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackwizInclude {
    pub location: Location,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Location {
    Path(PathBuf),
    Url(reqwest::Url),
}

impl IncludeType for MrpackInclude {
    const TYPE_NAME: &'static str = "mrpack";

    fn read(reader: &mut Reader) -> Self {
        Self {
            location: Location::read(reader),
            checksums: Checksums::read(reader),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        self.location.write(node);
        for (algorithm, digest) in self.checksums.iter() {
            node.push((algorithm.name(), digest));
        }
    }
}

impl IncludeType for PackwizInclude {
    const TYPE_NAME: &'static str = "packwiz";

    fn read(reader: &mut Reader) -> Self {
        Self {
            location: Location::read(reader),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        self.location.write(node);
    }
}

impl Location {
    fn read(reader: &mut Reader) -> Self {
        Self::from(reader.required_argument("pack location"))
    }

    fn write(&self, node: &mut KdlNode) {
        node.push(self.to_string());
    }
}

impl From<String> for Location {
    fn from(text: String) -> Self {
        match reqwest::Url::parse(&text) {
            Ok(url) if url.has_host() => Self::Url(url),
            _ => Self::Path(PathBuf::from(text)),
        }
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => f.debug_tuple("Path").field(path).finish(),
            Self::Url(url) => f.debug_tuple("Url").field(&url.as_str()).finish(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => f.write_str(&path.display().to_string()),
            Self::Url(url) => f.write_str(url.as_str()),
        }
    }
}

fn type_names() -> String {
    [MrpackInclude::TYPE_NAME, PackwizInclude::TYPE_NAME].join(", ")
}

impl Include {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Mrpack(_) => MrpackInclude::TYPE_NAME,
            Self::Packwiz(_) => PackwizInclude::TYPE_NAME,
        }
    }

    pub fn location(&self) -> &Location {
        match self {
            Self::Mrpack(include) => &include.location,
            Self::Packwiz(include) => &include.location,
        }
    }

    pub fn read(node: &KdlNode, errors: &mut Errors) -> Option<Spanned<Self>> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let type_name = reader.required_argument("pack format");

        let include = match type_name.as_str() {
            MrpackInclude::TYPE_NAME => Self::Mrpack(IncludeType::read(&mut reader)),
            PackwizInclude::TYPE_NAME => Self::Packwiz(IncludeType::read(&mut reader)),
            _ => {
                errors.push(
                    span,
                    format!(
                        "unknown pack format `{type_name}`, expected one of: {}",
                        type_names()
                    ),
                );
                return None;
            }
        };

        reader.reject_unread();
        Some(Spanned::new(include, span))
    }

    pub fn write(&self, node: &mut KdlNode) {
        node.push(self.type_name());

        match self {
            Self::Mrpack(include) => include.write(node),
            Self::Packwiz(include) => include.write(node),
        }
    }
}

impl Display for Include {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut node = KdlNode::new("include");
        self.write(&mut node);
        write_entries(&node, f)
    }
}
