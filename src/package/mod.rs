use knus::{
    ast::SpannedNode, decode::Context, errors::DecodeError, span::Span, Decode, DecodeChildren,
};
use miette::Result;

use crate::{
    core::kdl::{debug_without_span, decode_label, reject_beyond_label, reject_node},
    package::{artifact::PackageArtifact, build::PackageBuild, source::PackageSource},
};

pub mod artifact;
pub mod build;
pub mod source;

const PACKAGE_NODES: &str = "git, download, build, artifact";

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct Package {
    pub label: Option<String>,
    pub sources: Vec<PackageSource>,
    pub build: Option<PackageBuild>,
    pub artifacts: Vec<PackageArtifact>,
    pub span: Span,
}

debug_without_span!(Package {
    label,
    sources,
    build,
    artifacts,
});

impl Decode<Span> for Package {
    fn decode_node(
        node: &SpannedNode<Span>,
        ctx: &mut Context<Span>,
    ) -> Result<Self, DecodeError<Span>> {
        let label = decode_label(node, ctx);
        reject_beyond_label(node, ctx);

        let children = node
            .children
            .clone()
            .ok_or(DecodeError::missing(node, "package must have children"))?;

        let mut package = Self::decode_children(&children, ctx)?;
        package.label = label;
        package.span = *node.span();

        Ok(package)
    }
}

impl DecodeChildren<Span> for Package {
    fn decode_children(
        nodes: &[SpannedNode<Span>],
        ctx: &mut Context<Span>,
    ) -> Result<Self, DecodeError<Span>> {
        let mut package = Package::default();

        for node in nodes {
            match node.node_name.as_ref() {
                "git" | "download" => package.sources.push(PackageSource::decode_node(node, ctx)?),
                "artifact" => {
                    let artifact = PackageArtifact::decode_node(node, ctx)?;
                    if artifact.to.is_none() && artifact.from.file_name().is_none() {
                        ctx.emit_error(DecodeError::unexpected(
                            node,
                            "node",
                            "`artifact` needs a destination when its source has no file name",
                        ));
                    }
                    package.artifacts.push(artifact);
                }
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
