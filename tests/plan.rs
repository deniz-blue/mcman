use mcman::{
    addons::Platform,
    manifest::Manifest,
    plan::{self, Plan, PlanWarning},
};
use miette::Diagnostic;
use std::{fs, path::Path};

fn manifest(fixture: &str) -> Manifest {
    let path = format!("tests/fixtures/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    Manifest::parse(&path, &text).unwrap_or_else(|e| panic!("{path} should parse:\n{e:?}"))
}

fn plan(fixture: &str) -> Plan {
    plan::from_manifest(&manifest(fixture))
        .unwrap_or_else(|e| panic!("{fixture} should plan:\n{e:?}"))
}

fn addons(plan: &Plan, target: &str, directory: &str) -> Vec<String> {
    plan.targets
        .iter()
        .find(|candidate| candidate.target.name == target)
        .unwrap_or_else(|| panic!("no target `{target}` in the plan"))
        .directories
        .iter()
        .find(|candidate| candidate.path.as_deref() == Some(Path::new(directory)))
        .map(|found| found.addons.iter().map(|addon| addon.to_string()).collect())
        .unwrap_or_default()
}

fn platform(plan: &Plan, target: &str) -> Option<Platform> {
    plan.targets
        .iter()
        .find(|candidate| candidate.target.name == target)
        .unwrap_or_else(|| panic!("no target `{target}` in the plan"))
        .platform
        .as_ref()
        .map(|platform| platform.value.clone())
}

fn rejection(fixture: &str) -> String {
    plan::from_manifest(&manifest(fixture))
        .expect_err(&format!("{fixture} should be rejected"))
        .to_string()
}

fn labelled_source(fixture: &str) -> String {
    let error = plan::from_manifest(&manifest(fixture))
        .expect_err(&format!("{fixture} should be rejected"));
    let label = error
        .labels()
        .unwrap_or_else(|| panic!("{fixture} error carries no label"))
        .next()
        .unwrap_or_else(|| panic!("{fixture} error carries no label"));

    let text = fs::read_to_string(format!("tests/fixtures/{fixture}.kdl")).unwrap();
    text[label.offset()..label.offset() + label.len()].to_string()
}

#[test]
fn inheritance() {
    insta::assert_debug_snapshot!(plan("inheritance"));
}

#[test]
fn full_design_doc() {
    insta::assert_debug_snapshot!(plan("full-mcman"));
}

#[test]
fn an_ancestor_reaches_targets_in_subgroups() {
    let plan = plan("inheritance");

    assert!(addons(&plan, "lobby", "plugins").contains(&"modrinth:luckperms".into()));
    assert!(addons(&plan, "smp", "plugins").contains(&"modrinth:luckperms".into()));
    assert!(addons(&plan, "smp", "plugins").contains(&"modrinth:spark".into()));
}

#[test]
fn siblings_are_invisible_to_each_other() {
    let plan = plan("inheritance");
    let smp = addons(&plan, "smp", "plugins");

    assert!(
        !smp.iter()
            .any(|preset| preset.contains("fastasyncworldedit")),
        "smp picked up lobby's plugin: {smp:?}"
    );
}

#[test]
fn inherited_declarations_come_before_local_ones() {
    let plan = plan("inheritance");

    assert_eq!(
        addons(&plan, "lobby", "plugins"),
        [
            "modrinth:luckperms",
            "modrinth:spark",
            "modrinth:fastasyncworldedit",
        ]
    );
}


#[test]
fn two_declarations_writing_one_file_are_rejected() {
    let error = rejection("conflicting-file");

    assert!(
        error.contains("plugins/config.yml"),
        "unexpected error: {error}"
    );
}

#[test]
fn one_file_name_in_two_dirs_is_two_files() {
    let files = plan("dir-scoped-files");
    let directories = &files.targets[0].directories;

    assert_eq!(directories.len(), 2);
    for directory in directories {
        assert_eq!(directory.copies.len(), 1);
        assert_eq!(directory.copies[0].to, Path::new("config.yml"));
    }
}

#[test]
fn two_dirs_composing_to_one_path_are_rejected() {
    let error = rejection("conflicting-file-across-dirs");

    assert!(
        error.contains("plugins/LuckPerms/config.yml"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_group_reaching_no_target_warns() {
    assert_eq!(
        plan("inheritance").warnings,
        [PlanWarning::GroupWithoutTarget {
            label: Some("unreachable".into())
        }]
    );
}

#[test]
fn a_platform_reaches_targets_in_subgroups() {
    let plan = plan("platform");

    assert!(matches!(
        platform(&plan, "proxy"),
        Some(Platform::Velocity(_))
    ));
    assert_eq!(platform(&plan, "plain"), None);

    let Some(Platform::Fabric(fabric)) = platform(&plan, "smp") else {
        panic!("`smp` should inherit the fabric platform of its parent group");
    };
    assert_eq!(fabric.minecraft.as_deref(), Some("1.21.1"));
    assert_eq!(fabric.loader.as_deref(), Some("0.16.5"));
}

#[test]
fn a_redeclared_platform_is_rejected() {
    assert_eq!(
        rejection("redeclared-platform"),
        "a second platform `fabric` is declared in group `smp`"
    );
}

#[test]
fn a_redeclared_platform_points_at_both_declarations() {
    let error = plan::from_manifest(&manifest("redeclared-platform"))
        .expect_err("redeclared-platform should be rejected");
    let text = fs::read_to_string("tests/fixtures/redeclared-platform.kdl").unwrap();
    let sources: Vec<String> = error
        .labels()
        .expect("the error carries labels")
        .map(|label| text[label.offset()..label.offset() + label.len()].to_string())
        .collect();

    assert_eq!(
        sources,
        [
            "platform fabric minecraft=\"1.21.1\"",
            "platform paper minecraft=\"1.21.1\"",
        ]
    );
}

#[test]
fn a_repeated_target_name_is_rejected() {
    let error = rejection("duplicate-target");

    assert!(error.contains("`lobby`"), "unexpected error: {error}");
}

#[test]
fn a_redeclared_package_is_rejected() {
    let error = rejection("redeclared-package");

    assert!(
        error.contains("package `customplugin` is declared again in group `lobby`"),
        "unexpected error: {error}"
    );
}

#[test]
fn every_rejection_points_at_the_offending_declaration() {
    assert!(labelled_source("redeclared-package").contains("fork.git"));
    assert!(labelled_source("conflicting-file").contains("config/lobby.yml"));
    assert!(labelled_source("duplicate-target").contains("./run/elsewhere"));
}
