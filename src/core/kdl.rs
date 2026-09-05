use std::path::PathBuf;

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
pub struct DecodeError {
    pub message: String,
    #[label]
    pub at: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("could not read {}", .text.name())]
pub struct DecodeErrors {
    #[source_code]
    pub text: NamedSource<String>,
    #[related]
    pub errors: Vec<DecodeError>,
}

#[derive(Default)]
pub struct Errors(Vec<DecodeError>);

impl Errors {
    pub fn push(&mut self, at: SourceSpan, message: impl Into<String>) {
        self.0.push(DecodeError {
            message: message.into(),
            at,
        });
    }

    pub fn into_report(self, name: &str, text: &str) -> Option<DecodeErrors> {
        if self.0.is_empty() {
            return None;
        }

        Some(DecodeErrors {
            text: NamedSource::new(name, text.to_owned()).with_language("kdl"),
            errors: self.0,
        })
    }
}

pub struct Reader<'a> {
    node: &'a KdlNode,
    errors: &'a mut Errors,
    arguments_read: usize,
    properties_read: Vec<&'static str>,
}

impl<'a> Reader<'a> {
    pub fn new(node: &'a KdlNode, errors: &'a mut Errors) -> Self {
        Self {
            node,
            errors,
            arguments_read: 0,
            properties_read: Vec::new(),
        }
    }

    pub fn span(&self) -> SourceSpan {
        self.node.span()
    }

    pub fn argument(&mut self) -> Option<String> {
        let entry = self.arguments().nth(self.arguments_read)?;
        self.arguments_read += 1;
        self.string(entry)
    }

    pub fn required_argument(&mut self, what: &str) -> String {
        match self.argument() {
            Some(value) => value,
            None => {
                let span = self.node.span();
                self.errors.push(span, format!("`{what}` is required"));
                String::new()
            }
        }
    }

    pub fn path_argument(&mut self) -> Option<PathBuf> {
        self.argument().map(PathBuf::from)
    }

    pub fn required_path_argument(&mut self, what: &str) -> PathBuf {
        PathBuf::from(self.required_argument(what))
    }

    pub fn property(&mut self, name: &'static str) -> Option<String> {
        self.properties_read.push(name);
        let entry = self.node.entry(name)?;
        self.string(entry)
    }

    pub fn required_property(&mut self, name: &'static str) -> String {
        match self.property(name) {
            Some(value) => value,
            None => {
                let span = self.node.span();
                self.errors
                    .push(span, format!("property `{name}` is required"));
                String::new()
            }
        }
    }

    pub fn path_property(&mut self, name: &'static str) -> Option<PathBuf> {
        self.property(name).map(PathBuf::from)
    }

    pub fn integer_property(&mut self, name: &'static str) -> Option<i128> {
        self.properties_read.push(name);
        let entry = self.node.entry(name)?;
        match entry.value().as_integer() {
            Some(value) => Some(value),
            None => {
                self.errors
                    .push(entry.span(), format!("`{name}` must be a number"));
                None
            }
        }
    }

    pub fn flag_property(&mut self, name: &'static str) -> bool {
        self.properties_read.push(name);
        let Some(entry) = self.node.entry(name) else {
            return false;
        };

        match entry.value().as_bool() {
            Some(value) => value,
            None => {
                self.errors
                    .push(entry.span(), format!("`{name}` must be #true or #false"));
                false
            }
        }
    }

    pub fn required_children(&mut self, message: &str) -> bool {
        if self.node.children().is_some() {
            return true;
        }

        let span = self.node.span();
        self.errors.push(span, message);
        false
    }

    pub fn reject_unread(self) {
        let Self {
            node,
            errors,
            arguments_read,
            properties_read,
        } = self;
        let mut arguments_seen = 0;

        for entry in node.entries() {
            match entry.name() {
                Some(name) if properties_read.contains(&name.value()) => {}
                Some(name) => errors.push(
                    entry.span(),
                    format!("unexpected property `{}`", name.value()),
                ),
                None => {
                    arguments_seen += 1;
                    if arguments_seen > arguments_read {
                        errors.push(entry.span(), "unexpected argument");
                    }
                }
            }
        }
    }

    fn arguments(&self) -> impl Iterator<Item = &'a KdlEntry> {
        self.node
            .entries()
            .iter()
            .filter(|entry| entry.name().is_none())
    }

    fn string(&mut self, entry: &KdlEntry) -> Option<String> {
        match entry.value() {
            KdlValue::String(value) => Some(value.clone()),
            other => {
                self.errors
                    .push(entry.span(), format!("expected a string, found {other}"));
                None
            }
        }
    }
}

pub fn child_nodes(node: &KdlNode) -> &[KdlNode] {
    node.children().map(KdlDocument::nodes).unwrap_or_default()
}

pub fn reject_node(node: &KdlNode, errors: &mut Errors, allowed: &str) {
    let name = node.name().value();
    errors.push(
        node.span(),
        format!("unexpected node `{name}`, expected one of: {allowed}"),
    );
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
