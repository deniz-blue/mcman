use assert_cmd::Command;
use std::fs;

fn setup_fixture(fixture_name: &str) -> assert_fs::TempDir {
    let dir = assert_fs::TempDir::new().unwrap();
    let src = format!("tests/fixtures/{fixture_name}.kdl");
    fs::copy(&src, dir.path().join("mcman.kdl")).unwrap();
    dir
}

fn run_mcman(dir: &assert_fs::TempDir) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mcman")
        .unwrap()
        .current_dir(dir.path())
        .assert()
}

#[test]
fn parse_minimal() {
    let dir = setup_fixture("minimal");
    run_mcman(&dir).success();
}

#[test]
fn parse_server_with_presets() {
    let dir = setup_fixture("server-with-presets");
    run_mcman(&dir).success();
}

#[test]
fn parse_nested_groups() {
    let dir = setup_fixture("nested-groups");
    run_mcman(&dir).success();
}

#[test]
fn parse_multiple_targets() {
    let dir = setup_fixture("multiple-targets");
    run_mcman(&dir).success();
}

#[test]
fn parse_custom_package_download() {
    let dir = setup_fixture("custom-package-download");
    run_mcman(&dir).success();
}

#[test]
fn parse_custom_package_git_build() {
    let dir = setup_fixture("custom-package-git-build");
    run_mcman(&dir).success();
}

#[test]
fn parse_all_target_types() {
    let dir = setup_fixture("all-target-types");
    run_mcman(&dir).success();
}

#[test]
fn parse_empty_groups() {
    let dir = setup_fixture("empty-groups");
    run_mcman(&dir).success();
}

#[test]
fn parse_multi_dir() {
    let dir = setup_fixture("multi-dir");
    run_mcman(&dir).success();
}

#[test]
fn parse_full_design_doc() {
    let dir = setup_fixture("full-mcman");
    run_mcman(&dir).success();
}

#[test]
fn parse_package_with_path() {
    let dir = setup_fixture("package-with-path");
    run_mcman(&dir).success();
}

#[test]
fn parse_multiple_links() {
    let dir = setup_fixture("multiple-links");
    run_mcman(&dir).success();
}

// Error cases

#[test]
fn reject_missing_manifest() {
    let dir = assert_fs::TempDir::new().unwrap();
    // no mcman.kdl in the dir
    run_mcman(&dir).failure();
}
