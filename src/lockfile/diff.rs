use std::path::PathBuf;

use crate::{lockfile::Lockfile, plan::Plan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockChange {
    TargetAdded(String),
    TargetRemoved(String),
    TargetPathChanged {
        target: String,
        locked: PathBuf,
        wanted: PathBuf,
    },
    RuntimeAdded {
        target: String,
        identifier: String,
    },
    RuntimeRemoved {
        target: String,
        identifier: String,
    },
    PresetAdded {
        target: String,
        identifier: String,
    },
    PresetRemoved {
        target: String,
        identifier: String,
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

impl Lockfile {
    /// Membership only. Locked versions are resolved values, so comparing one to a
    /// manifest `version=` needs a provider.
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

            let wanted: Vec<&str> = planned
                .runtimes
                .iter()
                .map(|runtime| runtime.identifier.as_str())
                .collect();
            let held: Vec<&str> = locked
                .runtimes
                .iter()
                .map(|runtime| runtime.identifier.as_str())
                .collect();
            changes.extend(membership_changes(
                &wanted,
                &held,
                |identifier| LockChange::RuntimeAdded {
                    target: name.clone(),
                    identifier: identifier.to_owned(),
                },
                |identifier| LockChange::RuntimeRemoved {
                    target: name.clone(),
                    identifier: identifier.to_owned(),
                },
            ));

            let wanted: Vec<&str> = planned
                .directories
                .iter()
                .flat_map(|directory| &directory.presets)
                .map(|preset| preset.identifier.as_str())
                .collect();
            let held: Vec<&str> = locked
                .presets
                .iter()
                .map(|preset| preset.identifier.as_str())
                .collect();
            changes.extend(membership_changes(
                &wanted,
                &held,
                |identifier| LockChange::PresetAdded {
                    target: name.clone(),
                    identifier: identifier.to_owned(),
                },
                |identifier| LockChange::PresetRemoved {
                    target: name.clone(),
                    identifier: identifier.to_owned(),
                },
            ));

            let wanted: Vec<&str> = planned
                .directories
                .iter()
                .flat_map(|directory| &directory.packages)
                .filter_map(|package| package.label.as_deref())
                .collect();
            let held: Vec<&str> = locked
                .packages
                .iter()
                .map(|package| package.name.as_str())
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

fn membership_changes(
    wanted: &[&str],
    locked: &[&str],
    added: impl Fn(&str) -> LockChange,
    removed: impl Fn(&str) -> LockChange,
) -> Vec<LockChange> {
    let missing = wanted
        .iter()
        .filter(|name| !locked.contains(*name))
        .map(|name| added(name));
    let stale = locked
        .iter()
        .filter(|name| !wanted.contains(*name))
        .map(|name| removed(name));

    missing.chain(stale).collect()
}
