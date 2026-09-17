use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

use std::fmt::Display;

use miette::SourceSpan;

use crate::{
    addons::{Addon, Platform},
    core::kdl::Spanned,
    manifest::{Directory, Group, Include, Manifest, Target},
};

mod error;

pub use error::PlanError;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Plan {
    pub targets: Vec<TargetPlan>,
    pub warnings: Vec<PlanWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetPlan {
    pub target: Spanned<Target>,
    pub platforms: Vec<Spanned<Platform>>,
    pub runtimes: Vec<Spanned<Addon>>,
    pub includes: Vec<Spanned<Include>>,
    pub directories: Vec<Directory>,
    pub meta: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanWarning {
    GroupWithoutTarget { label: Option<String> },
}

impl Display for PlanWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GroupWithoutTarget { label: Some(label) } => {
                write!(f, "group `{label}` reaches no target")
            }
            Self::GroupWithoutTarget { label: None } => {
                f.write_str("the manifest declares no targets")
            }
        }
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
    platforms: Vec<Spanned<Platform>>,
    runtimes: Vec<Spanned<Addon>>,
    includes: Vec<Spanned<Include>>,
    directories: Vec<Directory>,
    meta: BTreeMap<String, String>,
    written_paths: HashSet<PathBuf>,
}

impl Scope {
    fn extend(&mut self, group: &Group) -> Result<(), PlanError> {
        let label = || group.label.clone();

        for platform in &group.platforms {
            let same_type = self
                .platforms
                .iter()
                .find(|declared| declared.type_name() == platform.type_name());

            if let Some(first) = same_type {
                return Err(PlanError::RedeclaredPlatform {
                    name: platform.type_name().to_owned(),
                    group: label(),
                    at: platform.span,
                    first: first.span,
                });
            }

            self.platforms.push(platform.clone());
        }

        self.runtimes.extend(group.runtimes.iter().cloned());
        self.includes.extend(group.includes.iter().cloned());
        self.meta.extend(group.meta.clone());

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

            merged.addons.extend(directory.addons.iter().cloned());

            for package in &directory.packages {
                if !insert_unique(&mut merged.packages, package, |a, b| a.label == b.label) {
                    return Err(PlanError::RedeclaredPackage {
                        label: package.label.clone(),
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

    fn missing_platform(&self) -> Option<PlanError> {
        for platform in &self.platforms {
            for required in platform.requires() {
                let declared = self
                    .platforms
                    .iter()
                    .any(|candidate| candidate.type_name() == *required);

                if !declared {
                    return Some(PlanError::MissingPlatform {
                        name: platform.type_name().to_owned(),
                        required: (*required).to_owned(),
                        at: platform.span,
                    });
                }
            }
        }

        None
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

    if !group.targets.is_empty() {
        if let Some(error) = scope.missing_platform() {
            return Err(error);
        }
    }

    for target in &group.targets {
        plan.targets.push(TargetPlan {
            target: target.clone(),
            platforms: scope.platforms.clone(),
            runtimes: scope.runtimes.clone(),
            includes: scope.includes.clone(),
            directories: scope.directories.clone(),
            meta: scope.meta.clone(),
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

fn canonical_directory_path(path: Option<&Path>) -> Option<PathBuf> {
    match path {
        Some(path) if path != Path::new(".") => Some(path.to_path_buf()),
        _ => None,
    }
}

fn insert_unique<T: Clone>(
    items: &mut Vec<T>,
    incoming: &T,
    same: impl Fn(&T, &T) -> bool,
) -> bool {
    if items.iter().any(|item| same(item, incoming)) {
        return false;
    }

    items.push(incoming.clone());
    true
}
