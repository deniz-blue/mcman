use std::ops::Deref;

use knus::{
    ast::{Literal, SpannedNode},
    decode::{Context, Kind},
    errors::DecodeError,
    traits::ErrorSpan,
};

/// Emits rather than returns, so one run reports every unknown node.
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

/// A hand-written `decode_node` reads argument 0 and nothing else; the rest is a typo.
pub fn reject_beyond_label<S: ErrorSpan>(node: &SpannedNode<S>, ctx: &mut Context<S>) {
    for argument in node.arguments.iter().skip(1) {
        ctx.emit_error(DecodeError::unexpected(
            &argument.literal,
            "argument",
            "unexpected argument",
        ));
    }

    for name in node.properties.keys() {
        ctx.emit_error(DecodeError::unexpected(
            name,
            "property",
            format!("unexpected property `{}`", name.as_ref()),
        ));
    }
}

/// Spans exist for diagnostics only. Omitting them from `Debug` keeps snapshots
/// structural, so editing one fixture does not shift every byte offset below it.
macro_rules! debug_without_span {
    ($type:ident { $($field:ident),* $(,)? }) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($type))
                    $(.field(stringify!($field), &self.$field))*
                    .finish()
            }
        }
    };
}

pub(crate) use debug_without_span;
