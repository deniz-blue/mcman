use mcman::manifest::Manifest;
use miette::Diagnostic;
use std::fs;

fn parse(fixture: &str) -> Manifest {
    let path = format!("tests/fixtures/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    Manifest::parse(&path, &text).unwrap_or_else(|e| panic!("{path} should parse:\n{e:?}"))
}

/// Collects the diagnostic messages alone. The rendered form embeds the
/// offending source lines, so matching against it would pass on any error
/// reported near them rather than on the one being asserted.
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

/// Snapshots the parsed tree, so a fixture that silently loses nodes fails
/// rather than passing because nothing errored.
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
}

rejects! {
    unknown_node: "unknown-node" => "unexpected node `plugins`",
    unknown_dir_node: "unknown-dir-node" => "unexpected node `download`",
    runtime_in_dir: "runtime-in-dir" => "unexpected node `runtime`",
    unknown_package_node: "unknown-package-node" => "unexpected node `link`",
    duplicate_build: "duplicate-build" => "a package may only have one `build`",
    unknown_target_type: "unknown-target-type" => "Invalid target type: kubernetes",
    group_without_children: "group-without-children" => "group must have children",
    surplus_group_entries: "surplus-group-entries" => "unexpected property `path`",
    surplus_package_entries: "surplus-package-entries" => "unexpected argument",
}
