use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use kdl::{KdlDocument, KdlNode};
use miette::Result;

use crate::{
    addons::{Addon, Platform},
    core::kdl::{child_nodes, reject_node, Errors, KdlMaybeRead, KdlRead, KdlWrite, Reader},
    manifest::{Include, TargetType},
    package::{source::download::Download, Package},
};

pub mod diff;

const LOCKFILE_NODES: &str = "lock, target";
const TARGET_NODES: &str = "meta, platform, runtime, use, include, package";
const PLATFORM_NODES: &str = "resolved";
const LOCKED_NODES: &str = "resolved, dir, download, artifact";
const PACKAGE_NODES: &str = "artifact";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Lockfile {
    pub header: LockfileHeader,
    pub targets: Vec<LockedTarget>,
}

impl Lockfile {
    pub fn parse(name: &str, text: &str) -> Result<Self> {
        let document: KdlDocument = text.parse().map_err(miette::Report::new)?;

        let mut errors = Errors::default();
        let mut lockfile = Self::default();

        for node in document.nodes() {
            match node.name().value() {
                "lock" => lockfile.header = LockfileHeader::read_node(node, &mut errors),
                "target" => lockfile.targets.push(LockedTarget::read(node, &mut errors)),
                _ => reject_node(node, &mut errors, LOCKFILE_NODES),
            }
        }

        match errors.into_report(name, text) {
            Some(report) => Err(report.into()),
            None => Ok(lockfile),
        }
    }

    pub fn to_kdl(&self) -> String {
        let mut document = KdlDocument::new();

        document.nodes_mut().push(self.header.to_kdl("lock"));

        for target in &self.targets {
            document.nodes_mut().push(target.to_kdl("target"));
        }

        document.autoformat();
        document.to_string()
    }
}

fn read_meta(node: &KdlNode, errors: &mut Errors) -> BTreeMap<String, String> {
    let mut reader = Reader::new(node, errors);
    let meta = reader.properties();
    reader.reject_unread();

    meta
}

fn push_child(node: &mut KdlNode, child: KdlNode) {
    node.ensure_children().nodes_mut().push(child);
}

fn push_artifacts(node: &mut KdlNode, artifacts: &[Artifact]) {
    for artifact in artifacts {
        push_child(node, artifact.to_kdl("artifact"));
    }
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockfileHeader {
    pub version: u64,
    pub generated: String,
}

impl KdlRead for LockfileHeader {
    fn read(reader: &mut Reader) -> Self {
        Self {
            version: reader.unsigned_property("version").unwrap_or_default(),
            generated: reader.required_property("generated"),
        }
    }
}

impl KdlWrite for LockfileHeader {
    fn write(&self, node: &mut KdlNode) {
        node.push(("version", i128::from(self.version)));
        node.push(("generated", self.generated.as_str()));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockedTarget {
    pub name: String,
    pub path: PathBuf,
    pub kind: TargetType,
    pub platforms: Vec<LockedPlatform>,
    pub runtimes: Vec<LockedAddon>,
    pub addons: Vec<LockedAddon>,
    pub includes: Vec<LockedInclude>,
    pub packages: Vec<LockedPackage>,
    pub meta: BTreeMap<String, String>,
}

impl LockedTarget {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let name = reader.required_argument("target name");
        let path = PathBuf::from(reader.required_property("path"));
        let kind = TargetType::read(&mut reader);
        reader.reject_unread();

        let mut target = Self {
            name,
            path,
            kind,
            ..Self::default()
        };

        for child in child_nodes(node) {
            match child.name().value() {
                "platform" => target.platforms.extend(LockedPlatform::read(child, errors)),
                "runtime" => target.runtimes.extend(LockedAddon::read(child, errors)),
                "use" => target.addons.extend(LockedAddon::read(child, errors)),
                "include" => target.includes.extend(LockedInclude::read(child, errors)),
                "package" => target.packages.push(LockedPackage::read(child, errors)),
                "meta" => target.meta.extend(read_meta(child, errors)),
                _ => reject_node(child, errors, TARGET_NODES),
            }
        }

        target
    }
}

impl KdlWrite for LockedTarget {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.name.as_str());
        node.push(("path", display(&self.path)));
        self.kind.write(node);

        if !self.meta.is_empty() {
            let mut meta = KdlNode::new("meta");
            for (key, value) in &self.meta {
                meta.push((key.as_str(), value.as_str()));
            }
            push_child(node, meta);
        }

        for platform in &self.platforms {
            push_child(node, platform.to_kdl("platform"));
        }

        for runtime in &self.runtimes {
            push_child(node, runtime.to_kdl("runtime"));
        }

        for addon in &self.addons {
            push_child(node, addon.to_kdl("use"));
        }

        for include in &self.includes {
            push_child(node, include.to_kdl("include"));
        }

        for package in &self.packages {
            push_child(node, package.to_kdl("package"));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPlatform {
    pub requested: Platform,
    pub resolved: Platform,
}

impl KdlMaybeRead for LockedPlatform {
    fn read(node: &KdlNode, errors: &mut Errors) -> Option<Self> {
        let requested = Platform::read(node, errors)?;
        let mut resolved = None;

        for child in child_nodes(node) {
            match child.name().value() {
                "resolved" => {
                    if resolved.is_some() {
                        errors.push(child.span(), "a locked `platform` has one `resolved`");
                    }
                    resolved = Platform::read(child, errors);
                }
                _ => reject_node(child, errors, PLATFORM_NODES),
            }
        }

        let Some(resolved) = resolved else {
            errors.push(node.span(), "a locked `platform` needs a `resolved` child");
            return None;
        };

        Some(Self {
            requested,
            resolved,
        })
    }
}

impl KdlWrite for LockedPlatform {
    fn write(&self, node: &mut KdlNode) {
        self.requested.write(node);

        push_child(node, self.resolved.to_kdl("resolved"));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locked<D> {
    pub requested: D,
    pub resolved: D,
    pub directory: PathBuf,
    pub package: Package,
    pub artifacts: Vec<Artifact>,
}

pub type LockedAddon = Locked<Addon>;
pub type LockedInclude = Locked<Include>;

impl<D: KdlMaybeRead + KdlWrite> KdlMaybeRead for Locked<D> {
    fn read(node: &KdlNode, errors: &mut Errors) -> Option<Self> {
        let requested = D::read(node, errors)?;

        let mut resolved = None;
        let mut directory = PathBuf::new();
        let mut downloads = Vec::new();
        let mut artifacts = Vec::new();

        for child in child_nodes(node) {
            match child.name().value() {
                "resolved" => {
                    if resolved.is_some() {
                        errors.push(child.span(), "a locked entry has one `resolved`");
                    }
                    resolved = D::read(child, errors);
                }
                "dir" => {
                    let mut reader = Reader::new(child, errors);
                    directory = reader.required_path_argument("directory path");
                    reader.reject_unread();
                }
                "download" => downloads.push(Download::read_node(child, errors)),
                "artifact" => artifacts.push(Artifact::read_node(child, errors)),
                _ => reject_node(child, errors, LOCKED_NODES),
            }
        }

        let Some(resolved) = resolved else {
            errors.push(node.span(), "a locked entry needs a `resolved` child");
            return None;
        };

        Some(Self {
            requested,
            resolved,
            directory,
            package: Package::from_downloads(downloads),
            artifacts,
        })
    }
}

impl<D: KdlWrite> KdlWrite for Locked<D> {
    fn write(&self, node: &mut KdlNode) {
        self.requested.write(node);

        push_child(node, self.resolved.to_kdl("resolved"));

        if !self.directory.as_os_str().is_empty() {
            push_child(node, self.directory.to_kdl("dir"));
        }

        self.package.write_sources(node);
        push_artifacts(node, &self.artifacts);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockedPackage {
    pub name: String,
    pub identity: String,
    pub artifacts: Vec<Artifact>,
}

impl LockedPackage {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let name = reader.required_argument("package name");
        let identity = reader.required_property("identity");
        reader.reject_unread();

        Self {
            name,
            identity,
            artifacts: read_artifacts(node, errors),
        }
    }
}

impl KdlWrite for LockedPackage {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.name.as_str());
        node.push(("identity", self.identity.as_str()));
        push_artifacts(node, &self.artifacts);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Artifact {
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
}

impl KdlRead for Artifact {
    fn read(reader: &mut Reader) -> Self {
        Self {
            path: reader.required_path_argument("artifact path"),
            hash: reader.required_property("hash"),
            size: reader.unsigned_property("size").unwrap_or_default(),
        }
    }
}

impl KdlWrite for Artifact {
    fn write(&self, node: &mut KdlNode) {
        node.push(display(&self.path));
        node.push(("hash", self.hash.as_str()));
        node.push(("size", i128::from(self.size)));
    }
}

fn read_artifacts(node: &KdlNode, errors: &mut Errors) -> Vec<Artifact> {
    let mut artifacts = Vec::new();

    for child in child_nodes(node) {
        match child.name().value() {
            "artifact" => artifacts.push(Artifact::read_node(child, errors)),
            _ => reject_node(child, errors, PACKAGE_NODES),
        }
    }

    artifacts
}
