use std::path::{Path, PathBuf};

use kdl::{KdlDocument, KdlNode};
use miette::Result;

use crate::{
    addons::{Addon, Platform},
    core::kdl::{child_nodes, reject_node, Errors, Reader},
    package::{source::download::Download, Package},
};

pub mod diff;

const LOCKFILE_NODES: &str = "meta, target";
const TARGET_NODES: &str = "platform, runtime, use, package";
const PLATFORM_NODES: &str = "resolved";
const ADDON_NODES: &str = "resolved, download, artifact";
const ENTRY_NODES: &str = "artifact";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Lockfile {
    pub meta: LockfileMeta,
    pub targets: Vec<LockedTarget>,
}

impl Lockfile {
    pub fn parse(name: &str, text: &str) -> Result<Self> {
        let document: KdlDocument = text.parse().map_err(miette::Report::new)?;

        let mut errors = Errors::default();
        let mut lockfile = Self::default();

        for node in document.nodes() {
            match node.name().value() {
                "meta" => lockfile.meta = LockfileMeta::read(node, &mut errors),
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

        let mut meta = KdlNode::new("meta");
        meta.push(("version", i128::from(self.meta.version)));
        meta.push(("generated", self.meta.generated.as_str()));
        document.nodes_mut().push(meta);

        for target in &self.targets {
            let mut node = KdlNode::new("target");
            node.push(target.name.as_str());
            node.push(("path", display(&target.path)));

            for platform in &target.platforms {
                node.ensure_children().nodes_mut().push(platform.to_kdl());
            }

            for runtime in &target.runtimes {
                node.ensure_children()
                    .nodes_mut()
                    .push(runtime.to_kdl("runtime"));
            }

            for addon in &target.addons {
                node.ensure_children().nodes_mut().push(addon.to_kdl("use"));
            }

            for package in &target.packages {
                let mut child = KdlNode::new("package");
                child.push(package.name.as_str());
                child.push(("identity", package.identity.as_str()));
                push_artifacts(&mut child, &package.artifacts);
                node.ensure_children().nodes_mut().push(child);
            }

            document.nodes_mut().push(node);
        }

        document.autoformat();
        document.to_string()
    }
}

fn push_artifacts(node: &mut KdlNode, artifacts: &[Artifact]) {
    for artifact in artifacts {
        let mut child = KdlNode::new("artifact");
        child.push(display(&artifact.path));
        child.push(("hash", artifact.hash.as_str()));
        child.push(("size", i128::from(artifact.size)));
        node.ensure_children().nodes_mut().push(child);
    }
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockfileMeta {
    pub version: u64,
    pub generated: String,
}

impl LockfileMeta {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let version = reader.unsigned_property("version").unwrap_or_default();
        let generated = reader.required_property("generated");
        reader.reject_unread();

        Self { version, generated }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockedTarget {
    pub name: String,
    pub path: PathBuf,
    pub platforms: Vec<LockedPlatform>,
    pub runtimes: Vec<LockedAddon>,
    pub addons: Vec<LockedAddon>,
    pub packages: Vec<LockedPackage>,
}

impl LockedTarget {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let name = reader.required_argument("target name");
        let path = PathBuf::from(reader.required_property("path"));
        reader.reject_unread();

        let mut target = Self {
            name,
            path,
            ..Self::default()
        };

        for child in child_nodes(node) {
            match child.name().value() {
                "platform" => target.platforms.extend(LockedPlatform::read(child, errors)),
                "runtime" => target.runtimes.extend(LockedAddon::read(child, errors)),
                "use" => target.addons.extend(LockedAddon::read(child, errors)),
                "package" => target.packages.push(LockedPackage::read(child, errors)),
                _ => reject_node(child, errors, TARGET_NODES),
            }
        }

        target
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPlatform {
    pub requested: Platform,
    pub resolved: Platform,
}

impl LockedPlatform {
    fn read(node: &KdlNode, errors: &mut Errors) -> Option<Self> {
        let requested = Platform::read(node, errors)?.value;
        let mut resolved = None;

        for child in child_nodes(node) {
            match child.name().value() {
                "resolved" => {
                    if resolved.is_some() {
                        errors.push(child.span(), "a locked `platform` has one `resolved`");
                    }
                    resolved = Platform::read(child, errors).map(|platform| platform.value);
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

    fn to_kdl(&self) -> KdlNode {
        let mut node = KdlNode::new("platform");
        self.requested.write(&mut node);

        let mut resolved = KdlNode::new("resolved");
        self.resolved.write(&mut resolved);
        node.ensure_children().nodes_mut().push(resolved);

        node
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedAddon {
    pub requested: Addon,
    pub resolved: Addon,
    pub package: Package,
    pub artifacts: Vec<Artifact>,
}

impl LockedAddon {
    fn read(node: &KdlNode, errors: &mut Errors) -> Option<Self> {
        let mut reader = Reader::new(node, errors);
        let requested = Addon::read(&mut reader)?;
        reader.reject_unread();

        let mut resolved = None;
        let mut downloads = Vec::new();
        let mut artifacts = Vec::new();

        for child in child_nodes(node) {
            match child.name().value() {
                "resolved" => {
                    if resolved.is_some() {
                        errors.push(child.span(), "a locked addon has one `resolved`");
                    }
                    let mut reader = Reader::new(child, errors);
                    resolved = Addon::read(&mut reader);
                    reader.reject_unread();
                }
                "download" => downloads.push(Download::read(child, errors)),
                "artifact" => artifacts.push(Artifact::read(child, errors)),
                _ => reject_node(child, errors, ADDON_NODES),
            }
        }

        let Some(resolved) = resolved else {
            errors.push(node.span(), "a locked addon needs a `resolved` child");
            return None;
        };

        Some(Self {
            requested,
            resolved,
            package: Package::from_downloads(downloads),
            artifacts,
        })
    }

    fn to_kdl(&self, name: &str) -> KdlNode {
        let mut node = KdlNode::new(name);
        self.requested.write(&mut node);

        let mut resolved = KdlNode::new("resolved");
        self.resolved.write(&mut resolved);
        node.ensure_children().nodes_mut().push(resolved);

        self.package.write_sources(&mut node);
        push_artifacts(&mut node, &self.artifacts);
        node
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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Artifact {
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
}

impl Artifact {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let path = reader.required_path_argument("artifact path");
        let hash = reader.required_property("hash");
        let size = reader.unsigned_property("size").unwrap_or_default();
        reader.reject_unread();

        Self { path, hash, size }
    }
}

fn read_artifacts(node: &KdlNode, errors: &mut Errors) -> Vec<Artifact> {
    let mut artifacts = Vec::new();

    for child in child_nodes(node) {
        match child.name().value() {
            "artifact" => artifacts.push(Artifact::read(child, errors)),
            _ => reject_node(child, errors, ENTRY_NODES),
        }
    }

    artifacts
}
