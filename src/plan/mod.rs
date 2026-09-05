use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::manifest::{Directory, Group, Manifest, Preset, Target};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Plan {
    pub targets: Vec<TargetPlan>,
    pub warnings: Vec<PlanWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetPlan {
    pub target: Target,
    pub runtimes: Vec<Preset>,
    pub directories: Vec<Directory>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanWarning {
    /// `label` is `None` for the manifest root.
    GroupWithoutTarget { label: Option<String> },
}

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

pub fn from_manifest(manifest: &Manifest) -> Result<Plan, PlanError> {
    let mut plan = Plan::default();
    walk(&manifest.root, &Scope::default(), &mut plan)?;

    let mut names = HashSet::new();
    for target in plan.targets.iter().map(|plan| &plan.target) {
        if !names.insert(&target.name) {
            return Err(PlanError::DuplicateTarget {
                name: target.name.clone(),
                at: target.span,
            });
        }
    }

    Ok(plan)
}

#[derive(Clone, Default)]
struct Scope {
    runtimes: Vec<Preset>,
    directories: Vec<Directory>,
    written_paths: HashSet<PathBuf>,
}

impl Scope {
    fn extend(&mut self, group: &Group) -> Result<(), PlanError> {
        let label = || group.label.clone();

        for runtime in &group.runtimes {
            if !insert_unique(&mut self.runtimes, runtime, |preset| &preset.identifier) {
                return Err(PlanError::RedeclaredPreset {
                    identifier: runtime.identifier.clone(),
                    group: label(),
                    at: runtime.span,
                });
            }
        }

        for directory in &group.directories {
            let path = canonical_directory_path(directory.path.as_deref());

            for copy in &directory.copies {
                let destination = written_path(path.as_deref(), &copy.to);
                self.claim_written_path(destination, group, copy.span)?;
            }

            for symlink in &directory.symlinks {
                let destination = written_path(path.as_deref(), &symlink.to);
                self.claim_written_path(destination, group, symlink.span)?;
            }

            let merged = self.directory(path);

            for preset in &directory.presets {
                if !insert_unique(&mut merged.presets, preset, |preset| &preset.identifier) {
                    return Err(PlanError::RedeclaredPreset {
                        identifier: preset.identifier.clone(),
                        group: label(),
                        at: preset.span,
                    });
                }
            }

            for package in &directory.packages {
                let Some(existing) = package.label.as_ref() else {
                    merged.packages.push(package.clone());
                    continue;
                };

                if !insert_unique(&mut merged.packages, package, |package| &package.label) {
                    return Err(PlanError::RedeclaredPackage {
                        label: existing.clone(),
                        group: label(),
                        at: package.span,
                    });
                }
            }

            merged.copies.extend(directory.copies.iter().cloned());
            merged.symlinks.extend(directory.symlinks.iter().cloned());
        }

        Ok(())
    }

    fn claim_written_path(
        &mut self,
        destination: PathBuf,
        group: &Group,
        at: SourceSpan,
    ) -> Result<(), PlanError> {
        if self.written_paths.insert(destination.clone()) {
            return Ok(());
        }

        Err(PlanError::ConflictingFile {
            destination,
            group: group.label.clone(),
            at,
        })
    }

    fn directory(&mut self, path: Option<PathBuf>) -> &mut Directory {
        match self
            .directories
            .iter()
            .position(|directory| directory.path == path)
        {
            Some(index) => &mut self.directories[index],
            None => {
                self.directories.push(Directory {
                    path,
                    ..Directory::default()
                });
                self.directories.last_mut().expect("just pushed")
            }
        }
    }
}

fn walk(group: &Group, inherited: &Scope, plan: &mut Plan) -> Result<(), PlanError> {
    let mut scope = inherited.clone();
    scope.extend(group)?;

    let targets_before = plan.targets.len();

    for target in &group.targets {
        plan.targets.push(TargetPlan {
            target: target.clone(),
            runtimes: scope.runtimes.clone(),
            directories: scope.directories.clone(),
        });
    }

    for subgroup in &group.subgroups {
        walk(subgroup, &scope, plan)?;
    }

    if plan.targets.len() == targets_before {
        plan.warnings.push(PlanWarning::GroupWithoutTarget {
            label: group.label.clone(),
        });
    }

    Ok(())
}

fn written_path(directory: Option<&Path>, destination: &Path) -> PathBuf {
    match directory {
        Some(directory) => directory.join(destination),
        None => destination.to_path_buf(),
    }
}

/// `dir "."` and a bare `use` both address the target root.
fn canonical_directory_path(path: Option<&Path>) -> Option<PathBuf> {
    match path {
        Some(path) if path != Path::new(".") => Some(path.to_path_buf()),
        _ => None,
    }
}

fn insert_unique<T: Clone, K: PartialEq>(
    items: &mut Vec<T>,
    incoming: &T,
    key: impl Fn(&T) -> &K,
) -> bool {
    if items.iter().any(|item| key(item) == key(incoming)) {
        return false;
    }

    items.push(incoming.clone());
    true
}
