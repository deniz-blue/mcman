use kdl::KdlNode;
use miette::Result;

use crate::{
    core::kdl::{child_nodes, reject_node, Errors, KdlRead, KdlWrite, Reader, Spanned},
    package::{
        artifact::PackageArtifact,
        build::PackageBuild,
        source::{download::Download, PackageSource},
    },
};

pub mod artifact;
pub mod build;
pub mod source;

const PACKAGE_NODES: &str = "git, download, build, artifact";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub label: Option<String>,
    pub sources: Vec<PackageSource>,
    pub build: Option<PackageBuild>,
    pub artifacts: Vec<PackageArtifact>,
}

impl Package {
    pub fn from_downloads(downloads: Vec<Download>) -> Self {
        let artifacts = downloads
            .iter()
            .map(|download| PackageArtifact {
                from: download.destination().to_owned(),
                to: None,
            })
            .collect();

        Self {
            label: None,
            sources: downloads.into_iter().map(PackageSource::Download).collect(),
            build: None,
            artifacts,
        }
    }

    pub fn write_sources(&self, node: &mut KdlNode) {
        for source in &self.sources {
            let child = source.to_kdl(source.node_name());
            node.ensure_children().nodes_mut().push(child);
        }
    }

    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Spanned<Self> {
        let mut reader = Reader::new(node, errors);
        let span = reader.span();
        let label = reader.argument();
        reader.required_children("package must have children");
        reader.reject_unread();

        let mut package = Self {
            label,
            sources: Vec::new(),
            build: None,
            artifacts: Vec::new(),
        };

        for child in child_nodes(node) {
            match child.name().value() {
                "git" | "download" => package.sources.push(PackageSource::read(child, errors)),
                "artifact" => {
                    let artifact = PackageArtifact::read_node(child, errors);
                    if artifact.to.is_none() && artifact.from.file_name().is_none() {
                        errors.push(
                            child.span(),
                            "`artifact` needs a destination when its source has no file name",
                        );
                    }
                    package.artifacts.push(artifact);
                }
                "build" => {
                    if package.build.is_some() {
                        errors.push(child.span(), "a package may only have one `build`");
                    }
                    package.build = Some(PackageBuild::read(child, errors));
                }
                _ => reject_node(child, errors, PACKAGE_NODES),
            }
        }

        Spanned::new(package, span)
    }

    pub async fn build(&self) -> Result<()> {
        Ok(())
    }
}
