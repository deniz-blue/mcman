use std::ops::Deref;

use knus::{Decode, DecodeChildren};

use crate::package::{build::PackageBuild, link::PackageLink, source::PackageSource};

pub mod build;
pub mod link;
pub mod source;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Package {
    pub label: Option<String>,
    pub sources: Vec<PackageSource>,
    pub build: Option<PackageBuild>,
    pub links: Vec<PackageLink>,
}

impl<S: knus::traits::ErrorSpan> Decode<S> for Package {
    fn decode_node(
        node: &knus::ast::SpannedNode<S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        let label = node.arguments.get(0).map(|arg| match arg.literal.deref() {
            knus::ast::Literal::String(s) => s.clone().into(),
            _ => String::new(),
        });

        let mut package = Self::decode_children(
            &node
                .children
                .clone()
                .ok_or(knus::errors::DecodeError::Missing {
                    span: node.span().clone(),
                    message: String::from("package must have children"),
                })?,
            ctx,
        )?;

        package.label = label;

        Ok(package)
    }
}

impl<S: knus::traits::ErrorSpan> DecodeChildren<S> for Package {
    fn decode_children(
        nodes: &[knus::ast::SpannedNode<S>],
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        let mut package = Package::default();

        for node in nodes {
            match &*node.node_name.as_ref() {
                "build" => package.build = Some(PackageBuild::decode_node(node, ctx)?),
                "link" => package.links.push(PackageLink::decode_node(node, ctx)?),
                _ => {
                    let source = PackageSource::decode_node(node, ctx)?;
                    package.sources.push(source);
                }
            }
        }

        Ok(package)
    }
}
