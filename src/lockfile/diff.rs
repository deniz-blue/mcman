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
    PlatformChanged {
        target: String,
        locked: Option<String>,
        wanted: Option<String>,
    },
    RuntimeAdded {
        target: String,
        identifier: String,
    },
    RuntimeRemoved {
        target: String,
        identifier: String,
    },
    AddonAdded {
        target: String,
        addon: String,
    },
    AddonRemoved {
        target: String,
        addon: String,
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

            let locked_platform = locked
                .platform
                .as_ref()
                .map(|platform| platform.name.as_str());
            let wanted_platform = planned
                .platform
                .as_ref()
                .map(|platform| platform.type_name());

            if locked_platform != wanted_platform {
                changes.push(LockChange::PlatformChanged {
                    target: name.clone(),
                    locked: locked_platform.map(str::to_owned),
                    wanted: wanted_platform.map(str::to_owned),
                });
            }

            let wanted: Vec<String> = planned
                .runtimes
                .iter()
                .map(|addon| addon.to_string())
                .collect();
            let held: Vec<String> = locked
                .runtimes
                .iter()
                .map(|runtime| runtime.identifier.clone())
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

            let wanted: Vec<String> = planned
                .directories
                .iter()
                .flat_map(|directory| &directory.addons)
                .map(|addon| addon.to_string())
                .collect();
            let held: Vec<String> = locked
                .addons
                .iter()
                .map(|addon| addon.identifier.clone())
                .collect();
            changes.extend(membership_changes(
                &wanted,
                &held,
                |addon| LockChange::AddonAdded {
                    target: name.clone(),
                    addon: addon.to_owned(),
                },
                |addon| LockChange::AddonRemoved {
                    target: name.clone(),
                    addon: addon.to_owned(),
                },
            ));

            let wanted: Vec<String> = planned
                .directories
                .iter()
                .flat_map(|directory| &directory.packages)
                .filter_map(|package| package.label.clone())
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

fn membership_changes(
    wanted: &[String],
    locked: &[String],
    added: impl Fn(&str) -> LockChange,
    removed: impl Fn(&str) -> LockChange,
) -> Vec<LockChange> {
    let missing = wanted
        .iter()
        .filter(|name| !locked.contains(name))
        .map(|name| added(name));
    let stale = locked
        .iter()
        .filter(|name| !wanted.contains(name))
        .map(|name| removed(name));

    missing.chain(stale).collect()
}
