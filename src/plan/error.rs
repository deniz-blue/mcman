use std::path::PathBuf;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum PlanError {
    #[error("`{identifier}` is declared again in {}", describe(.group))]
    #[diagnostic(
        code(mcman::redeclared_preset),
        help("A package may be declared once on the path from the root to a target. To give some targets a different version, move the package into a group that only those targets are under.")
    )]
    RedeclaredPreset {
        identifier: String,
        group: Option<String>,
        #[label("declared again here")]
        at: SourceSpan,
    },

    #[error("package `{label}` is declared again in {}", describe(.group))]
    #[diagnostic(
        code(mcman::redeclared_package),
        help("A package may be declared once on the path from the root to a target.")
    )]
    RedeclaredPackage {
        label: String,
        group: Option<String>,
        #[label("declared again here")]
        at: SourceSpan,
    },

    #[error("`{}` is written by more than one `fs:copy` or `fs:symlink` in {}", .destination.display(), describe(.group))]
    #[diagnostic(
        code(mcman::conflicting_file),
        help("Two declarations on the same path would race to write the same file.")
    )]
    ConflictingFile {
        destination: PathBuf,
        group: Option<String>,
        #[label("also written here")]
        at: SourceSpan,
    },

    #[error("a second platform `{name}` is declared in {}", describe(.group))]
    #[diagnostic(
        code(mcman::redeclared_platform),
        help("A target runs on one platform. To give some targets a different one, move it into a group that only those targets are under.")
    )]
    RedeclaredPlatform {
        name: String,
        group: Option<String>,
        #[label("declared again here")]
        at: SourceSpan,
        #[label("already declared here")]
        first: SourceSpan,
    },

    #[error("target `{name}` is declared more than once")]
    #[diagnostic(
        code(mcman::duplicate_target),
        help("Target names key the lockfile, so each must name exactly one output.")
    )]
    DuplicateTarget {
        name: String,
        #[label("declared again here")]
        at: SourceSpan,
    },
}

fn describe(group: &Option<String>) -> String {
    match group {
        Some(label) => format!("group `{label}`"),
        None => String::from("the manifest root"),
    }
}
