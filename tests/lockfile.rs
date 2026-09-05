use mcman::lockfile::Lockfile;
use std::{fs, path::Path};

fn lockfile(fixture: &str) -> Lockfile {
    let path = format!("tests/fixtures/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    Lockfile::parse(&path, &text).unwrap_or_else(|e| panic!("{path} should parse:\n{e:?}"))
}

#[test]
fn artifacts_carry_path_hash_and_size() {
    let lock = lockfile("lock-artifacts");
    let target = &lock.targets[0];

    let preset = &target.presets[0].artifacts[0];
    assert_eq!(preset.path, Path::new("velocity-3.4.0.jar"));
    assert_eq!(preset.hash, "abcdef");
    assert_eq!(preset.size, 123_456);

    let package = &target.packages[0].artifacts[0];
    assert_eq!(package.path, Path::new("plugins/customplugin.jar"));
    assert_eq!(package.size, 654_321);
}
