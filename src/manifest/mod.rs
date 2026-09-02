use std::{path::PathBuf, str::FromStr};

use knus::{
    ast::SpannedNode, decode::Context, errors::DecodeError, traits::ErrorSpan, Decode,
    DecodeChildren,
};
use miette::{miette, Result};

use crate::{
    core::kdl::{decode_label, reject_beyond_label, reject_node},
    package::Package,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Manifest {
    pub root: Group,
}

impl Manifest {
    pub fn parse(name: &str, text: &str) -> Result<Self> {
        knus::parse(name, text).map_err(Into::into)
    }
}

impl<S: ErrorSpan> DecodeChildren<S> for Manifest {
    fn decode_children(
        nodes: &[SpannedNode<S>],
        ctx: &mut Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        Ok(Self {
            root: Group::decode_children(nodes, ctx)?,
        })
    }
}

const GROUP_NODES: &str = "group, dir, target, use, package, runtime, copy, link";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Group {
    pub label: Option<String>,
    pub runtimes: Vec<Preset>,
    pub directories: Vec<Directory>,
    pub copies: Vec<CopyFile>,
    pub links: Vec<LinkFile>,
    pub targets: Vec<Target>,
    pub subgroups: Vec<Group>,
}

impl<S: ErrorSpan> Decode<S> for Group {
    fn decode_node(node: &SpannedNode<S>, ctx: &mut Context<S>) -> Result<Self, DecodeError<S>> {
        let label = decode_label(node, ctx);
        reject_beyond_label(node, ctx);

        let children = node
            .children
            .clone()
            .ok_or(DecodeError::missing(node, "group must have children"))?;

        let mut group = Self::decode_children(&children, ctx)?;
        group.label = label;

        Ok(group)
    }
}

impl<S: ErrorSpan> DecodeChildren<S> for Group {
    fn decode_children(
        nodes: &[SpannedNode<S>],
        ctx: &mut Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let mut group = Group::default();
        // `use` and `package` written outside any `dir` place files at the target
        // root, which is a `dir` with no path.
        let mut target_root = Directory::default();

        for node in nodes {
            match &*node.node_name.as_ref() {
                "dir" => group.directories.push(Directory::decode_node(node, ctx)?),
                "target" => group.targets.push(Target::decode_node(node, ctx)?),
                "group" => group.subgroups.push(Group::decode_node(node, ctx)?),
                "runtime" => group.runtimes.push(Preset::decode_node(node, ctx)?),
                "copy" => group.copies.push(CopyFile::decode_node(node, ctx)?),
                "link" => group.links.push(LinkFile::decode_node(node, ctx)?),
                "use" => target_root.presets.push(Preset::decode_node(node, ctx)?),
                "package" => target_root.packages.push(Package::decode_node(node, ctx)?),
                _ => reject_node(node, ctx, GROUP_NODES),
            }
        }

        if !target_root.presets.is_empty() || !target_root.packages.is_empty() {
            group.directories.push(target_root);
        }

        Ok(group)
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

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct CopyFile {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: PathBuf,
    #[knus(property, default)]
    pub overwrite: bool,
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct LinkFile {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: PathBuf,
}
