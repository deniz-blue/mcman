use mcman::{
    lockfile::{diff::LockChange, Lockfile},
    manifest::Manifest,
    plan::{self, Plan},
};
use std::{fs, path::Path};

fn lockfile(fixture: &str) -> Lockfile {
    let path = format!("tests/fixtures/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    Lockfile::parse(&path, &text).unwrap_or_else(|e| panic!("{path} should parse:\n{e:?}"))
}

fn plan(fixture: &str) -> Plan {
    let path = format!("tests/fixtures/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let manifest =
        Manifest::parse(&path, &text).unwrap_or_else(|e| panic!("{path} should parse:\n{e:?}"));

    plan::from_manifest(&manifest).unwrap_or_else(|e| panic!("{path} should plan:\n{e:?}"))
}

#[test]
fn artifacts_carry_path_hash_and_size() {
    let lock = lockfile("lock-artifacts");
    let target = &lock.targets[0];

    let addon = &target.addons[0].artifacts[0];
    assert_eq!(addon.path, Path::new("velocity-3.4.0.jar"));
    assert_eq!(addon.hash, "abcdef");
    assert_eq!(addon.size, 123_456);

    let package = &target.packages[0].artifacts[0];
    assert_eq!(package.path, Path::new("plugins/customplugin.jar"));
    assert_eq!(package.size, 654_321);
}

#[test]
fn writing_and_reparsing_returns_the_same_lockfile() {
    let lock = lockfile("lock-full");

    assert_eq!(
        Lockfile::parse("rewritten.kdl", &lock.to_kdl()).unwrap(),
        lock
    );
}

#[test]
fn a_written_lockfile_reads_like_the_design_doc() {
    insta::assert_snapshot!(lockfile("lock-full").to_kdl());
}

#[test]
fn a_lock_covering_its_manifest_needs_no_changes() {
    let changes = lockfile("lock-diff").changes_needed_for(&plan("lock-diff-manifest"));

    assert_eq!(changes, []);
}

#[test]
fn a_lock_for_other_targets_reports_both_directions() {
    let changes = lockfile("lock-artifacts").changes_needed_for(&plan("lock-diff-manifest"));

    assert_eq!(
        changes,
        [
            LockChange::TargetAdded("smp".into()),
            LockChange::TargetRemoved("proxy".into()),
        ]
    );
}

#[test]
fn a_package_missing_from_the_lock_is_reported() {
    let mut lock = lockfile("lock-diff");
    lock.targets[0].packages.clear();

    assert_eq!(
        lock.changes_needed_for(&plan("lock-diff-manifest")),
        [LockChange::PackageAdded {
            target: "smp".into(),
            label: "customplugin".into(),
        }]
    );
}

#[test]
fn a_moved_target_is_reported() {
    let mut lock = lockfile("lock-diff");
    lock.targets[0].path = Path::new("./run/elsewhere").to_path_buf();

    assert_eq!(
        lock.changes_needed_for(&plan("lock-diff-manifest")),
        [LockChange::TargetPathChanged {
            target: "smp".into(),
            locked: Path::new("./run/elsewhere").to_path_buf(),
            wanted: Path::new("./run/smp").to_path_buf(),
        }]
    );
}

#[test]
fn a_changed_platform_is_reported() {
    let mut lock = lockfile("lock-diff");
    lock.targets[0]
        .platform
        .as_mut()
        .expect("lock-diff locks a platform")
        .name = "fabric".into();

    assert_eq!(
        lock.changes_needed_for(&plan("lock-diff-manifest")),
        [LockChange::PlatformChanged {
            target: "smp".into(),
            locked: Some("fabric".into()),
            wanted: Some("paper".into()),
        }]
    );
}

#[test]
fn a_numeric_hash_survives_the_round_trip() {
    let mut lock = lockfile("lock-full");
    lock.targets[0].addons[0].artifacts[0].hash = "123456".into();
    lock.targets[0].name = "12345".into();

    assert_eq!(
        Lockfile::parse("rewritten.kdl", &lock.to_kdl()).unwrap(),
        lock
    );
}
