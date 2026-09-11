use std::{collections::BTreeMap, fmt, ops::Deref, path::PathBuf};

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Clone, Copy)]
pub struct Spanned<T> {
    pub value: T,
    pub span: SourceSpan,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: SourceSpan) -> Self {
        Self { value, span }
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Spanned<T> {}

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
    properties_read: Vec<&'a str>,
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

    pub fn reject(&mut self, message: String) {
        let span = self.node.span();
        self.errors.push(span, message);
    }

    pub fn argument(&mut self) -> Option<String> {
        let entry = self.arguments().nth(self.arguments_read)?;
        self.arguments_read += 1;
        self.string(entry)
    }

    pub fn required_argument(&mut self, what: &str) -> String {
        let written = self.arguments().nth(self.arguments_read).is_some();

        match self.argument() {
            Some(value) => value,
            None => {
                if !written {
                    let span = self.node.span();
                    self.errors.push(span, format!("`{what}` is required"));
                }
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
        let written = self.node.entry(name).is_some();

        match self.property(name) {
            Some(value) => value,
            None => {
                if !written {
                    let span = self.node.span();
                    self.errors
                        .push(span, format!("property `{name}` is required"));
                }
                String::new()
            }
        }
    }

    pub fn properties(&mut self) -> BTreeMap<String, String> {
        let node = self.node;
        let mut properties = BTreeMap::new();

        for entry in node.entries() {
            let Some(name) = entry.name() else { continue };
            self.properties_read.push(name.value());

            if let Some(value) = self.string(entry) {
                properties.insert(name.value().to_owned(), value);
            }
        }

        properties
    }

    pub fn path_property(&mut self, name: &'static str) -> Option<PathBuf> {
        self.property(name).map(PathBuf::from)
    }

    pub fn unsigned_property(&mut self, name: &'static str) -> Option<u64> {
        self.properties_read.push(name);
        let entry = self.node.entry(name)?;

        let Some(value) = entry.value().as_integer() else {
            self.errors
                .push(entry.span(), format!("`{name}` must be a number"));
            return None;
        };

        if value < 0 {
            self.errors
                .push(entry.span(), format!("`{name}` cannot be negative"));
            return None;
        }

        match u64::try_from(value) {
            Ok(value) => Some(value),
            Err(_) => {
                self.errors
                    .push(entry.span(), format!("`{name}` is too large"));
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
