use std::fmt::Display;

use kdl::KdlNode;
use thiserror::Error;

use crate::{
    addons::{fabric::FabricAddon, modrinth::ModrinthAddon, papermc::PaperMcAddon},
    core::kdl::{Errors, Reader, Spanned},
};

pub mod fabric;
pub mod modrinth;
pub mod papermc;
pub mod platform;
pub mod version;

pub use platform::{Platform, PlatformType};
pub use version::{RequestedVersion, Stability};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Addon {
    Modrinth(ModrinthAddon),
    PaperMc(PaperMcAddon),
    Fabric(FabricAddon),
}

pub trait AddonType: Display + Sized {
    const TYPE_NAME: &'static str;

    fn read(name: &str, reader: &mut Reader) -> Self;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AddonError {
    #[error("`{0}` is not written as `<type>:<name>`")]
    Unqualified(String),

    #[error("unknown addon type `{type_name}`, expected one of: {}", type_names())]
    UnknownType { type_name: String },

    #[error("`{0}` has no addon name after the `:`")]
    Nameless(String),
}

fn type_names() -> String {
    [
        ModrinthAddon::TYPE_NAME,
        PaperMcAddon::TYPE_NAME,
        FabricAddon::TYPE_NAME,
    ]
    .join(", ")
}

impl Addon {
    pub fn from_identifier(identifier: &str, reader: &mut Reader) -> Result<Self, AddonError> {
        let Some((type_name, name)) = identifier.split_once(':') else {
            return Err(AddonError::Unqualified(identifier.to_owned()));
        };

        if name.is_empty() {
            return Err(AddonError::Nameless(identifier.to_owned()));
        }

        Ok(match type_name {
            ModrinthAddon::TYPE_NAME => Self::Modrinth(ModrinthAddon::read(name, reader)),
            PaperMcAddon::TYPE_NAME => Self::PaperMc(PaperMcAddon::read(name, reader)),
            FabricAddon::TYPE_NAME => Self::Fabric(FabricAddon::read(name, reader)),
            _ => {
                return Err(AddonError::UnknownType {
                    type_name: type_name.to_owned(),
                })
            }
        })
    }

    pub fn read(node: &KdlNode, errors: &mut Errors) -> Option<Spanned<Self>> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let identifier = reader.required_argument("addon identifier");

        match Self::from_identifier(&identifier, &mut reader) {
            Ok(addon) => {
                reader.reject_unread();
                Some(Spanned::new(addon, span))
            }
            Err(error) => {
                errors.push(span, error.to_string());
                None
            }
        }
    }
}

impl Display for Addon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modrinth(addon) => addon.fmt(f),
            Self::PaperMc(addon) => addon.fmt(f),
            Self::Fabric(addon) => addon.fmt(f),
        }
    }
}
