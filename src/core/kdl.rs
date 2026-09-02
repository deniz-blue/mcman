use std::ops::Deref;

use knus::{
    ast::{Literal, SpannedNode},
    decode::{Context, Kind},
    errors::DecodeError,
    traits::ErrorSpan,
};

/// Records `node` as not belonging where it was found, and keeps decoding so
/// every unknown node in the manifest is reported at once rather than one per run.
pub fn reject_node<S: ErrorSpan>(node: &SpannedNode<S>, ctx: &mut Context<S>, allowed: &str) {
    let name = node.node_name.as_ref();
    ctx.emit_error(DecodeError::unexpected(
        node,
        "node",
        format!("unexpected node `{name}`, expected one of: {allowed}"),
    ));
}

pub fn decode_label<S: ErrorSpan>(node: &SpannedNode<S>, ctx: &mut Context<S>) -> Option<String> {
    let argument = node.arguments.first()?;
    match argument.literal.deref() {
        Literal::String(s) => Some(s.to_string()),
        _ => {
            ctx.emit_error(DecodeError::scalar_kind(Kind::String, &argument.literal));
            None
        }
    }
}

/// A hand-written `decode_node` reads a label and nothing else, so every
/// surplus argument and every property on such a node is a typo.
pub fn reject_beyond_label<S: ErrorSpan>(node: &SpannedNode<S>, ctx: &mut Context<S>) {
    for argument in node.arguments.iter().skip(1) {
        ctx.emit_error(DecodeError::unexpected(
            &argument.literal,
            "argument",
            "unexpected argument",
        ));
    }

    for (name, _) in node.properties.iter() {
        ctx.emit_error(DecodeError::unexpected(
            name,
            "property",
            format!("unexpected property `{}`", name.as_ref()),
        ));
    }
}
