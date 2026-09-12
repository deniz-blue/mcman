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
    let declaration = "use modrinth luckperms version=\"latest\"\n";
    let one = Manifest::parse("one.kdl", declaration).expect("should parse");
    let two = Manifest::parse("two.kdl", &format!("\n\n{declaration}")).expect("should parse");

    assert_eq!(one, two);
}

#[test]
fn each_addon_type_writes_its_own_label() {
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
            "modrinth:spark",
            "hangar:ViaVersion",
            "curseforge:238222",
            "github:PaperMC/Velocity",
            "maven:com.example:mylib",
            "fabric",
            "modrinth:create",
            "modrinth:a:b",
            "papermc:paper",
        ]
    );
}

parses! {
    all_target_types: "all-target-types",
    empty_groups: "empty-groups",
    multi_dir: "multi-dir",
    full_design_doc: "full-mcman",
    package_with_path: "package-with-path",
    multiple_artifacts: "multiple-artifacts",
    root_files: "root-files",
    platform: "platform",
    addon: "addon",
    download_checksums: "download-checksums",
}

rejects! {
    unknown_node: "unknown-node" => "unexpected node `plugins`",
    duplicate_build: "duplicate-build" => "a package may only have one `build`",
    group_without_children: "group-without-children" => "group must have children",
    surplus_package_entries: "surplus-package-entries" => "unexpected argument",
    artifact_without_file_name: "artifact-without-file-name" => "`artifact` needs a destination when its source has no file name",
    platform_property_is_not_a_string: "platform-property-is-not-a-string" => "expected a string, found 1.21",
    unknown_platform: "unknown-platform" => "unknown platform `papr`, expected one of: paper, velocity, fabric",
    platform_with_unknown_property: "platform-with-unknown-property" => "unexpected property `minecarft`",
    addon_unknown_type: "addon-unknown-type" => "unknown addon type `modrnth`, expected one of: modrinth, papermc, fabric, hangar, curseforge, github, maven",
    addon_without_an_id: "addon-without-an-id" => "`id` is required",
    addon_id_given_twice: "addon-id-given-twice" => "`id` is given twice",
    checksum_wrong_length: "checksum-wrong-length" => "`sha512` is 128 hex characters, found 6",
    negative_size: "negative-size" => "`size` cannot be negative",
    download_without_a_file_name: "download-without-a-file-name" => "`download` needs a `path` when its url has no file name",
    execute_unbalanced_quote: "execute-unbalanced-quote" => "has an unbalanced quote",
    execute_without_a_command: "execute-without-a-command" => "`execute` needs a command to run",
}
