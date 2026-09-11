use mcman::manifest::Manifest;
use std::{fs, path::Path};

fn parse(fixture: &str) -> Manifest {
    let path = format!("tests/fixtures/{fixture}.kdl");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    Manifest::parse(&path, &text).unwrap_or_else(|e| panic!("{path} should parse:\n{e:?}"))
}

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

#[test]
fn the_same_declaration_at_two_offsets_is_equal() {
    let declaration = "use \"modrinth:luckperms\" version=\"latest\"\n";
    let one = Manifest::parse("one.kdl", declaration).expect("should parse");
    let two = Manifest::parse("two.kdl", &format!("\n\n{declaration}")).expect("should parse");

    assert_eq!(one, two);
}

#[test]
fn an_addon_writes_back_the_identifier_it_was_read_from() {
    let manifest = parse("addon");
    let written: Vec<String> = manifest
        .root
        .directories
        .iter()
        .flat_map(|dir| &dir.addons)
        .map(|addon| addon.to_string())
        .collect();

    assert_eq!(
        written,
        [
            "modrinth:luckperms",
            "fabric:fabric",
            "modrinth:create",
            "modrinth:sodium",
            "modrinth:a:b",
            "papermc:paper",
        ]
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
    root_files: "root-files",
    dir_scoped_files: "dir-scoped-files",
    artifact_default_destination: "artifact-default-destination",
    platform: "platform",
    addon: "addon",
    download_checksums: "download-checksums",
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
    platform_in_dir: "platform-in-dir" => "unexpected node `platform`",
    platform_property_is_not_a_string: "platform-property-is-not-a-string" => "expected a string, found 1.21",
    unknown_platform: "unknown-platform" => "unknown platform `papr`, expected one of: paper, velocity, fabric",
    platform_without_minecraft: "platform-without-minecraft" => "property `minecraft` is required",
    platform_with_unknown_property: "platform-with-unknown-property" => "unexpected property `minecarft`",
    addon_unqualified: "addon-unqualified" => "`luckperms` is not written as `<type>:<name>`",
    addon_unknown_type: "addon-unknown-type" => "unknown addon type `modrnth`, expected one of: modrinth, papermc, fabric",
    addon_without_a_name: "addon-without-a-name" => "`modrinth:` has no addon name after the `:`",
    checksum_wrong_length: "checksum-wrong-length" => "`sha512` is 128 hex characters, found 6",
    build_with_entries: "build-with-entries" => "unexpected property `bogus`",
    negative_size: "negative-size" => "`size` cannot be negative",
    size_past_u64: "size-past-u64" => "`size` is too large",
    download_without_a_file_name: "download-without-a-file-name" => "`download` needs a `path` when its url has no file name",
    execute_unbalanced_quote: "execute-unbalanced-quote" => "has an unbalanced quote",
    execute_without_a_command: "execute-without-a-command" => "`execute` needs a command to run",
}
