use std::path::{Path, PathBuf};

use kdl::{KdlDocument, KdlNode};
use miette::Result;

use crate::core::kdl::{child_nodes, reject_node, Errors, Reader};

pub mod diff;

const LOCKFILE_NODES: &str = "meta, target";
const TARGET_NODES: &str = "runtime, use, package";
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

            for runtime in &target.runtimes {
                let mut child = KdlNode::new("runtime");
                child.push(runtime.identifier.as_str());
                child.push(("version", runtime.version.as_str()));
                node.ensure_children().nodes_mut().push(child);
            }

            for preset in &target.presets {
                let mut child = KdlNode::new("use");
                child.push(preset.identifier.as_str());
                child.push(("version", preset.version.as_str()));
                push_artifacts(&mut child, &preset.artifacts);
                node.ensure_children().nodes_mut().push(child);
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
    pub version: u8,
    pub generated: String,
}

impl LockfileMeta {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let version = reader.integer_property("version").unwrap_or_default() as u8;
        let generated = reader.required_property("generated");
        reader.reject_unread();

        Self { version, generated }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockedTarget {
    pub name: String,
    pub path: PathBuf,
    pub runtimes: Vec<LockedRuntime>,
    pub presets: Vec<LockedPreset>,
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
                "runtime" => target.runtimes.push(LockedRuntime::read(child, errors)),
                "use" => target.presets.push(LockedPreset::read(child, errors)),
                "package" => target.packages.push(LockedPackage::read(child, errors)),
                _ => reject_node(child, errors, TARGET_NODES),
            }
        }

        target
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockedRuntime {
    pub identifier: String,
    pub version: String,
}

impl LockedRuntime {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let identifier = reader.required_argument("runtime identifier");
        let version = reader.required_property("version");
        reader.reject_unread();

        Self {
            identifier,
            version,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LockedPreset {
    pub identifier: String,
    pub version: String,
    pub artifacts: Vec<Artifact>,
}

impl LockedPreset {
    fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let identifier = reader.required_argument("preset identifier");
        let version = reader.required_property("version");
        reader.reject_unread();

        Self {
            identifier,
            version,
            artifacts: read_artifacts(node, errors),
        }
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
        let size = reader.integer_property("size").unwrap_or_default() as u64;
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
