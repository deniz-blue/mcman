use std::fmt::Display;

use kdl::KdlNode;

use crate::core::{
    checksum::Checksums,
    kdl::{write_entries, Declaration, Errors, Reader, Spanned},
    location::Location,
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

impl IncludeType for MrpackInclude {
    const TYPE_NAME: &'static str = "mrpack";

    fn read(reader: &mut Reader) -> Self {
        Self {
            location: Location::from(reader.required_argument("pack location")),
            checksums: Checksums::read(reader),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        node.push(self.location.to_string());
        for (algorithm, digest) in self.checksums.iter() {
            node.push((algorithm.name(), digest));
        }
    }
}

impl IncludeType for PackwizInclude {
    const TYPE_NAME: &'static str = "packwiz";

    fn read(reader: &mut Reader) -> Self {
        Self {
            location: Location::from(reader.required_argument("pack location")),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        node.push(self.location.to_string());
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
}

impl Declaration for Include {
    fn read(node: &KdlNode, errors: &mut Errors) -> Option<Spanned<Self>> {
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

    fn write(&self, node: &mut KdlNode) {
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
