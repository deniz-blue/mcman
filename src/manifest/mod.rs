use std::path::PathBuf;

use kdl::{KdlDocument, KdlNode};
use miette::Result;

use crate::{
    addons::{Addon, Platform},
    core::kdl::{child_nodes, reject_node, Errors, Reader, Spanned},
    package::Package,
};

mod fs;
mod target;

pub use fs::{CopyFile, SymlinkFile};
pub use target::{Target, TargetType};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Manifest {
    pub root: Group,
}

impl Manifest {
    pub fn parse(name: &str, text: &str) -> Result<Self> {
        let document: KdlDocument = text.parse().map_err(miette::Report::new)?;

        let mut errors = Errors::default();
        let root = Group::read(document.nodes(), &mut errors);

        match errors.into_report(name, text) {
            Some(report) => Err(report.into()),
            None => Ok(Self { root }),
        }
    }
}

const GROUP_NODES: &str =
    "group, dir, target, use, package, runtime, platform, fs:copy, fs:symlink";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Group {
    pub label: Option<String>,
    pub platforms: Vec<Spanned<Platform>>,
    pub runtimes: Vec<Spanned<Addon>>,
    pub directories: Vec<Directory>,
    pub targets: Vec<Spanned<Target>>,
    pub subgroups: Vec<Group>,
}

impl Group {
    fn read_node(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let label = reader.argument();
        let has_children = reader.required_children("group must have children");
        reader.reject_unread();

        if !has_children {
            return Self {
                label,
                ..Self::default()
            };
        }

        let children = child_nodes(node);
        let mut group = Self::read(children, errors);
        group.label = label;
        group
    }

    fn read(nodes: &[KdlNode], errors: &mut Errors) -> Self {
        let mut group = Group::default();
        let mut target_root = Directory::default();

        for node in nodes {
            match node.name().value() {
                "dir" => group.directories.push(Directory::read(node, errors)),
                "target" => group.targets.push(Target::read(node, errors)),
                "group" => group.subgroups.push(Group::read_node(node, errors)),
                "runtime" => group.runtimes.extend(Addon::read(node, errors)),
                "platform" => group.platforms.extend(Platform::read(node, errors)),
                "use" => target_root.addons.extend(Addon::read(node, errors)),
                "package" => target_root.packages.push(Package::read(node, errors)),
                "fs:copy" => target_root.copies.push(CopyFile::read(node, errors)),
                "fs:symlink" => target_root.symlinks.push(SymlinkFile::read(node, errors)),
                _ => reject_node(node, errors, GROUP_NODES),
            }
        }

        if !target_root.is_empty() {
            group.directories.push(target_root);
        }

        group
    }
}

const DIR_NODES: &str = "use, package, fs:copy, fs:symlink";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Directory {
    pub path: Option<PathBuf>,
    pub addons: Vec<Spanned<Addon>>,
    pub packages: Vec<Spanned<Package>>,
    pub copies: Vec<Spanned<CopyFile>>,
    pub symlinks: Vec<Spanned<SymlinkFile>>,
}

impl Directory {
    pub fn is_empty(&self) -> bool {
        self.addons.is_empty()
            && self.packages.is_empty()
            && self.copies.is_empty()
            && self.symlinks.is_empty()
    }

    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let path = reader.path_argument();
        reader.reject_unread();

        let mut directory = Self {
            path,
            ..Self::default()
        };

        for child in child_nodes(node) {
            match child.name().value() {
                "use" => directory.addons.extend(Addon::read(child, errors)),
                "package" => directory.packages.push(Package::read(child, errors)),
                "fs:copy" => directory.copies.push(CopyFile::read(child, errors)),
                "fs:symlink" => directory.symlinks.push(SymlinkFile::read(child, errors)),
                _ => reject_node(child, errors, DIR_NODES),
            }
        }

        directory
    }
}
