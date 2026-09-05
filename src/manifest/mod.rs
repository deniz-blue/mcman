use std::{path::PathBuf, str::FromStr};

use knus::{
    ast::SpannedNode, decode::Context, errors::DecodeError, span::Span, Decode, DecodeChildren,
};
use miette::{miette, Result};

use crate::{
    core::kdl::{debug_without_span, decode_label, reject_beyond_label, reject_node},
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

impl DecodeChildren<Span> for Manifest {
    fn decode_children(
        nodes: &[SpannedNode<Span>],
        ctx: &mut Context<Span>,
    ) -> Result<Self, DecodeError<Span>> {
        Ok(Self {
            root: Group::decode_children(nodes, ctx)?,
        })
    }
}

const GROUP_NODES: &str = "group, dir, target, use, package, runtime, fs:copy, fs:symlink";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Group {
    pub label: Option<String>,
    pub runtimes: Vec<Preset>,
    pub directories: Vec<Directory>,
    pub targets: Vec<Target>,
    pub subgroups: Vec<Group>,
}

impl Decode<Span> for Group {
    fn decode_node(
        node: &SpannedNode<Span>,
        ctx: &mut Context<Span>,
    ) -> Result<Self, DecodeError<Span>> {
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

impl DecodeChildren<Span> for Group {
    fn decode_children(
        nodes: &[SpannedNode<Span>],
        ctx: &mut Context<Span>,
    ) -> Result<Self, DecodeError<Span>> {
        let mut group = Group::default();
        // The target root is a `dir` with no path.
        let mut target_root = Directory::default();

        for node in nodes {
            match node.node_name.as_ref() {
                "dir" => group.directories.push(Directory::decode_node(node, ctx)?),
                "target" => group.targets.push(Target::decode_node(node, ctx)?),
                "group" => group.subgroups.push(Group::decode_node(node, ctx)?),
                "runtime" => group.runtimes.push(Preset::decode_node(node, ctx)?),
                "use" => target_root.presets.push(Preset::decode_node(node, ctx)?),
                "package" => target_root.packages.push(Package::decode_node(node, ctx)?),
                "fs:copy" => target_root.copies.push(CopyFile::decode_node(node, ctx)?),
                "fs:symlink" => target_root
                    .symlinks
                    .push(SymlinkFile::decode_node(node, ctx)?),
                _ => reject_node(node, ctx, GROUP_NODES),
            }
        }

        if !target_root.is_empty() {
            group.directories.push(target_root);
        }

        Ok(group)
    }
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct Directory {
    #[knus(argument)]
    pub path: Option<PathBuf>,
    #[knus(children(name = "use"))]
    pub presets: Vec<Preset>,
    #[knus(children(name = "package"))]
    pub packages: Vec<Package>,
    #[knus(children(name = "fs:copy"))]
    pub copies: Vec<CopyFile>,
    #[knus(children(name = "fs:symlink"))]
    pub symlinks: Vec<SymlinkFile>,
}

impl Directory {
    pub fn is_empty(&self) -> bool {
        self.presets.is_empty()
            && self.packages.is_empty()
            && self.copies.is_empty()
            && self.symlinks.is_empty()
    }
}

#[derive(Decode, Clone, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct Target {
    #[knus(argument)]
    pub name: String,
    #[knus(property, str)]
    pub path: Option<PathBuf>,
    #[knus(property(name = "type"), str)]
    pub kind: TargetType,
    #[knus(span)]
    pub span: Span,
}

debug_without_span!(Target { name, path, kind });

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
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

#[derive(Decode, Clone, PartialEq, Eq, Hash)]
#[knus(span_type = knus::span::Span)]
pub struct Preset {
    #[knus(argument)]
    pub identifier: String,
    #[knus(property)]
    pub version: Option<String>,
    #[knus(span)]
    pub span: Span,
}

debug_without_span!(Preset {
    identifier,
    version
});

#[derive(Decode, Clone, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct CopyFile {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: PathBuf,
    #[knus(property, default)]
    pub overwrite: bool,
    #[knus(span)]
    pub span: Span,
}

debug_without_span!(CopyFile {
    from,
    to,
    overwrite
});

#[derive(Decode, Clone, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct SymlinkFile {
    #[knus(argument)]
    pub from: PathBuf,
    #[knus(argument)]
    pub to: PathBuf,
    #[knus(span)]
    pub span: Span,
}

debug_without_span!(SymlinkFile { from, to });
