use mcman::manifest::Manifest;
use std::{fs, path::Path};

fn parse(fixture: &str) -> Manifest {
    let path = format!("tests/fixtures/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    Manifest::parse(&path, &text).unwrap_or_else(|e| panic!("{path} should parse:\n{e:?}"))
}

/// Messages only — the rendered form embeds source lines, so matching against it
/// would pass on any error reported near them.
fn rejection(fixture: &str) -> String {
    let path = format!("tests/fixtures/invalid/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let report = match Manifest::parse(&path, &text) {
        Ok(manifest) => panic!("{path} should have been rejected, parsed as:\n{manifest:#?}"),
        Err(report) => report,
    };

    let mut messages = vec![report.to_string()];
    if let Some(related) = report.related() {
        messages.extend(related.map(|diagnostic| diagnostic.to_string()));
    }
    messages.join("\n")
}

macro_rules! parses {
    ($($test:ident: $fixture:literal,)*) => {
        $(
            #[test]
            fn $test() {
                insta::assert_debug_snapshot!(parse($fixture));
            }
        )*
    };
}

macro_rules! rejects {
    ($($test:ident: $fixture:literal => $needle:literal,)*) => {
        $(
            #[test]
            fn $test() {
                let error = rejection($fixture);
                assert!(
                    error.contains($needle),
                    "expected an error mentioning {:?}, got:\n{error}",
                    $needle,
                );
            }
        )*
    };
}

#[test]
fn a_one_argument_artifact_takes_the_source_file_name() {
    let manifest = parse("artifact-default-destination");
    let directory = &manifest.root.directories[0];

    assert_eq!(
        directory.packages[0].artifacts[0].destination(),
        Path::new("custom-1.0.jar")
    );
}

parses! {
    minimal: "minimal",
    server_with_presets: "server-with-presets",
    nested_groups: "nested-groups",
    multiple_targets: "multiple-targets",
    custom_package_download: "custom-package-download",
    custom_package_git_build: "custom-package-git-build",
    all_target_types: "all-target-types",
    empty_groups: "empty-groups",
    multi_dir: "multi-dir",
    full_design_doc: "full-mcman",
    package_with_path: "package-with-path",
    multiple_artifacts: "multiple-artifacts",
    runtime_and_files: "runtime-and-files",
    dir_scoped_files: "dir-scoped-files",
    artifact_default_destination: "artifact-default-destination",
}

rejects! {
    unknown_node: "unknown-node" => "unexpected node `plugins`",
    unknown_dir_node: "unknown-dir-node" => "unexpected node `download`",
    runtime_in_dir: "runtime-in-dir" => "unexpected node `runtime`",
    unknown_package_node: "unknown-package-node" => "unexpected node `fs:symlink`",
    duplicate_build: "duplicate-build" => "a package may only have one `build`",
    unknown_target_type: "unknown-target-type" => "Invalid target type: kubernetes",
    group_without_children: "group-without-children" => "group must have children",
    surplus_group_entries: "surplus-group-entries" => "unexpected property `path`",
    surplus_package_entries: "surplus-package-entries" => "unexpected argument",
    artifact_without_file_name: "artifact-without-file-name" => "`artifact` needs a destination when its source has no file name",
}
