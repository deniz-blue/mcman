use std::{fmt::Display, path::PathBuf};

use crate::{
    addons::{Addon, Platform},
    lockfile::Lockfile,
    manifest::{Include, TargetType},
    plan::Plan,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockChange {
    TargetAdded(String),
    TargetRemoved(String),
    TargetPathChanged {
        target: String,
        locked: PathBuf,
        wanted: PathBuf,
    },
    TargetTypeChanged {
        target: String,
        locked: TargetType,
        wanted: TargetType,
    },
    PlatformAdded {
        target: String,
        platform: Platform,
    },
    PlatformRemoved {
        target: String,
        platform: Platform,
    },
    PlatformChanged {
        target: String,
        locked: Platform,
        wanted: Platform,
    },
    RuntimeAdded {
        target: String,
        runtime: Addon,
    },
    RuntimeRemoved {
        target: String,
        runtime: Addon,
    },
    AddonAdded {
        target: String,
        addon: Addon,
    },
    AddonRemoved {
        target: String,
        addon: Addon,
    },
    IncludeAdded {
        target: String,
        include: Include,
    },
    IncludeRemoved {
        target: String,
        include: Include,
    },
    PackageAdded {
        target: String,
        label: String,
    },
    PackageRemoved {
        target: String,
        label: String,
    },
}

impl Display for LockChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetAdded(target) => write!(f, "target `{target}` added"),
            Self::TargetRemoved(target) => write!(f, "target `{target}` removed"),
            Self::TargetPathChanged {
                target,
                locked,
                wanted,
            } => write!(
                f,
                "target `{target}` moved from `{}` to `{}`",
                locked.display(),
                wanted.display()
            ),
            Self::TargetTypeChanged {
                target,
                locked,
                wanted,
            } => write!(
                f,
                "target `{target}` type changed from `{locked}` to `{wanted}`"
            ),
            Self::PlatformAdded { target, platform } => {
                write!(f, "target `{target}` platform `{platform}` added")
            }
            Self::PlatformRemoved { target, platform } => {
                write!(f, "target `{target}` platform `{platform}` removed")
            }
            Self::PlatformChanged {
                target,
                locked,
                wanted,
            } => write!(
                f,
                "target `{target}` platform changed from `{locked}` to `{wanted}`"
            ),
            Self::RuntimeAdded { target, runtime } => {
                write!(f, "target `{target}` runtime `{runtime}` added")
            }
            Self::RuntimeRemoved { target, runtime } => {
                write!(f, "target `{target}` runtime `{runtime}` removed")
            }
            Self::AddonAdded { target, addon } => write!(f, "target `{target}` `{addon}` added"),
            Self::AddonRemoved { target, addon } => {
                write!(f, "target `{target}` `{addon}` removed")
            }
            Self::IncludeAdded { target, include } => {
                write!(f, "target `{target}` include `{include}` added")
            }
            Self::IncludeRemoved { target, include } => {
                write!(f, "target `{target}` include `{include}` removed")
            }
            Self::PackageAdded { target, label } => {
                write!(f, "target `{target}` package `{label}` added")
            }
            Self::PackageRemoved { target, label } => {
                write!(f, "target `{target}` package `{label}` removed")
            }
        }
    }
}

impl Lockfile {
    pub fn changes_needed_for(&self, plan: &Plan) -> Vec<LockChange> {
        let mut changes = Vec::new();

        for planned in &plan.targets {
            let name = &planned.target.name;

            let Some(locked) = self.targets.iter().find(|target| &target.name == name) else {
                changes.push(LockChange::TargetAdded(name.clone()));
                continue;
            };

            let wanted_path = planned
                .target
                .path
                .clone()
                .unwrap_or_else(|| PathBuf::from("."));

            if locked.path != wanted_path {
                changes.push(LockChange::TargetPathChanged {
                    target: name.clone(),
                    locked: locked.path.clone(),
                    wanted: wanted_path,
                });
            }

            if locked.kind != planned.target.kind {
                changes.push(LockChange::TargetTypeChanged {
                    target: name.clone(),
                    locked: locked.kind,
                    wanted: planned.target.kind,
                });
            }

            for wanted in planned.platforms.iter().map(|platform| &platform.value) {
                let held = locked
                    .platforms
                    .iter()
                    .map(|platform| &platform.requested)
                    .find(|platform| platform.type_name() == wanted.type_name());

                match held {
                    None => changes.push(LockChange::PlatformAdded {
                        target: name.clone(),
                        platform: wanted.clone(),
                    }),
                    Some(held) if held != wanted => changes.push(LockChange::PlatformChanged {
                        target: name.clone(),
                        locked: held.clone(),
                        wanted: wanted.clone(),
                    }),
                    Some(_) => {}
                }
            }

            for held in locked.platforms.iter().map(|platform| &platform.requested) {
                let still_wanted = planned
                    .platforms
                    .iter()
                    .any(|platform| platform.value.type_name() == held.type_name());

                if !still_wanted {
                    changes.push(LockChange::PlatformRemoved {
                        target: name.clone(),
                        platform: held.clone(),
                    });
                }
            }

            let wanted: Vec<Addon> = planned
                .runtimes
                .iter()
                .map(|runtime| runtime.value.clone())
                .collect();
            let held: Vec<Addon> = locked
                .runtimes
                .iter()
                .map(|runtime| runtime.requested.clone())
                .collect();
            changes.extend(membership_changes(
                &wanted,
                &held,
                |runtime| LockChange::RuntimeAdded {
                    target: name.clone(),
                    runtime: runtime.clone(),
                },
                |runtime| LockChange::RuntimeRemoved {
                    target: name.clone(),
                    runtime: runtime.clone(),
                },
            ));

            let wanted: Vec<Addon> = planned
                .directories
                .iter()
                .flat_map(|directory| &directory.addons)
                .map(|addon| addon.value.clone())
                .collect();
            let held: Vec<Addon> = locked
                .addons
                .iter()
                .map(|addon| addon.requested.clone())
                .collect();
            changes.extend(membership_changes(
                &wanted,
                &held,
                |addon| LockChange::AddonAdded {
                    target: name.clone(),
                    addon: addon.clone(),
                },
                |addon| LockChange::AddonRemoved {
                    target: name.clone(),
                    addon: addon.clone(),
                },
            ));

            let wanted: Vec<Include> = planned
                .includes
                .iter()
                .map(|include| include.value.clone())
                .collect();
            let held: Vec<Include> = locked
                .includes
                .iter()
                .map(|include| include.requested.clone())
                .collect();
            changes.extend(membership_changes(
                &wanted,
                &held,
                |include| LockChange::IncludeAdded {
                    target: name.clone(),
                    include: include.clone(),
                },
                |include| LockChange::IncludeRemoved {
                    target: name.clone(),
                    include: include.clone(),
                },
            ));

            let wanted: Vec<String> = planned
                .directories
                .iter()
                .flat_map(|directory| &directory.packages)
                .map(|package| package.label.clone())
                .collect();
            let held: Vec<String> = locked
                .packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            changes.extend(membership_changes(
                &wanted,
                &held,
                |label| LockChange::PackageAdded {
                    target: name.clone(),
                    label: label.to_owned(),
                },
                |label| LockChange::PackageRemoved {
                    target: name.clone(),
                    label: label.to_owned(),
                },
            ));
        }

        for locked in &self.targets {
            if !plan
                .targets
                .iter()
                .any(|planned| planned.target.name == locked.name)
            {
                changes.push(LockChange::TargetRemoved(locked.name.clone()));
            }
        }

        changes
    }
}

fn membership_changes<T: PartialEq>(
    wanted: &[T],
    locked: &[T],
    added: impl Fn(&T) -> LockChange,
    removed: impl Fn(&T) -> LockChange,
) -> Vec<LockChange> {
    let missing = wanted
        .iter()
        .filter(|name| !locked.contains(name))
        .map(added);
    let stale = locked
        .iter()
        .filter(|name| !wanted.contains(name))
        .map(removed);

    missing.chain(stale).collect()
}
