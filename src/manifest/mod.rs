use std::{ops::Deref, path::PathBuf, str::FromStr};

use knus::{Decode, DecodeChildren};
use miette::miette;

use crate::package::Package;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Manifest {
    pub root: Group,
}

impl<S: knus::traits::ErrorSpan> DecodeChildren<S> for Manifest {
    fn decode_children(
        nodes: &[knus::ast::SpannedNode<S>],
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        Ok(Self {
            root: Group::decode_children(nodes, ctx)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Group {
    pub label: Option<String>,
    pub directories: Vec<Directory>,
    pub targets: Vec<Target>,
    pub subgroups: Vec<Group>,
}

impl<S: knus::traits::ErrorSpan> Decode<S> for Group {
    fn decode_node(
        node: &knus::ast::SpannedNode<S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        let label = node.arguments.get(0).map(|arg| match arg.literal.deref() {
            knus::ast::Literal::String(s) => s.clone().into(),
            _ => String::new(),
        });

        let mut group = Self::decode_children(
            &node
                .children
                .clone()
                .ok_or(knus::errors::DecodeError::Missing {
                    span: node.span().clone(),
                    message: String::from("group must have children"),
                })?,
            ctx,
        )?;

        group.label = label;
        
		Ok(group)
    }
}

impl<S: knus::traits::ErrorSpan> DecodeChildren<S> for Group {
    fn decode_children(
        nodes: &[knus::ast::SpannedNode<S>],
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        let mut directories = Vec::new();
        let mut targets = Vec::new();
        let mut subgroups = Vec::new();
        let mut rest = Vec::new();

        for node in nodes {
            match &*node.node_name.as_ref() {
                "dir" => directories.push(Directory::decode_node(node, ctx)?),
                "target" => targets.push(Target::decode_node(node, ctx)?),
                "group" => subgroups.push(Group::decode_node(node, ctx)?),
                _ => rest.push(node.clone()),
            }
        }

        if !rest.is_empty() {
            directories.push(Directory::decode_children(&rest, ctx)?);
        }

        Ok(Self {
            label: None,
            directories,
            targets,
            subgroups,
        })
    }
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Directory {
    #[knus(argument)]
    pub path: Option<PathBuf>,
    #[knus(children(name = "use"))]
    pub presets: Vec<Preset>,
    #[knus(children(name = "package"))]
    pub packages: Vec<Package>,
}

impl<S: knus::traits::ErrorSpan> DecodeChildren<S> for Directory {
    fn decode_children(
        nodes: &[knus::ast::SpannedNode<S>],
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        let mut presets = Vec::new();
        let mut packages = Vec::new();

        for node in nodes {
            match &*node.node_name.as_ref() {
                "use" => presets.push(Preset::decode_node(node, ctx)?),
                "package" => packages.push(Package::decode_node(node, ctx)?),
                _ => {}
            }
        }

        Ok(Self {
            path: None,
            presets,
            packages,
        })
    }
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

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Preset {
    #[knus(argument)]
    pub identifier: String,
    #[knus(property)]
    pub version: Option<String>,
}
