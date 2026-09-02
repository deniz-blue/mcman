use knus::{
    ast::SpannedNode, decode::Context, errors::DecodeError, traits::ErrorSpan, Decode,
    DecodeChildren,
};
use miette::Result;

use crate::{
    core::kdl::{decode_label, reject_beyond_label, reject_node},
    package::{artifact::PackageArtifact, build::PackageBuild, source::PackageSource},
};

pub mod artifact;
pub mod build;
pub mod source;

const PACKAGE_NODES: &str = "git, download, build, artifact";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Package {
    pub label: Option<String>,
    pub sources: Vec<PackageSource>,
    pub build: Option<PackageBuild>,
    pub artifacts: Vec<PackageArtifact>,
}

impl<S: ErrorSpan> Decode<S> for Package {
    fn decode_node(node: &SpannedNode<S>, ctx: &mut Context<S>) -> Result<Self, DecodeError<S>> {
        let label = decode_label(node, ctx);
        reject_beyond_label(node, ctx);

        let children = node
            .children
            .clone()
            .ok_or(DecodeError::missing(node, "package must have children"))?;

        let mut package = Self::decode_children(&children, ctx)?;
        package.label = label;

        Ok(package)
    }
}

impl<S: ErrorSpan> DecodeChildren<S> for Package {
    fn decode_children(
        nodes: &[SpannedNode<S>],
        ctx: &mut Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let mut package = Package::default();

        for node in nodes {
            match &*node.node_name.as_ref() {
                "git" | "download" => {
                    package.sources.push(PackageSource::decode_node(node, ctx)?)
                }
                "artifact" => package
                    .artifacts
                    .push(PackageArtifact::decode_node(node, ctx)?),
                "build" => {
                    if package.build.is_some() {
                        ctx.emit_error(DecodeError::unexpected(
                            node,
                            "node",
                            "a package may only have one `build`",
                        ));
                    }
                    package.build = Some(PackageBuild::decode_node(node, ctx)?);
                }
                _ => reject_node(node, ctx, PACKAGE_NODES),
            }
        }

        Ok(package)
    }
}

impl Package {
    pub async fn build(&self) -> Result<()> {
        Ok(())
    }
}
