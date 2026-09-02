use assert_cmd::Command;

#[test]
fn reports_a_missing_manifest() {
    let dir = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("mcman")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("reading manifest"));
}
