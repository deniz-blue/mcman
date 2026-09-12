use mcman::{
    addons::{modrinth::ModrinthAddon, platform::paper::PaperPlatform, Platform},
    package::source::PackageSource,
    providers::modrinth::{
        loaders_for, select_files, select_version, to_resolved, ModrinthDependencyType,
        ModrinthProject, ModrinthSideSupport, ModrinthVersion, ModrinthVersionQuery,
        ModrinthVersionType,
    },
};

fn fixture<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let path = format!("tests/fixtures/modrinth/{name}.json");
    let text = std::fs::read_to_string(&path).expect(&path);
    serde_json::from_str(&text).expect(&path)
}

#[test]
fn project_reads_side_support() {
    let project: ModrinthProject = fixture("project");

    assert_eq!(project.id.0, "Vebnzrzj");
    assert_eq!(project.slug, "luckperms");
    assert_eq!(project.server_side, ModrinthSideSupport::Required);
    assert_eq!(project.client_side, ModrinthSideSupport::Unsupported);
    assert!(project.loaders.contains(&"paper".to_owned()));
}

#[test]
fn version_reads_its_primary_file() {
    let versions: Vec<ModrinthVersion> = fixture("versions");
    let version = &versions[0];
    let file = version.primary_file().expect("the fixture has one file");

    assert_eq!(version.version_number, "v5.5.71-bukkit");
    assert_eq!(version.version_type, ModrinthVersionType::Release);
    assert_eq!(file.filename, "LuckPerms-Bukkit-5.5.71.jar");
    assert_eq!(file.size, 1501521);
    assert_eq!(file.hashes.sha1, "fc8bb9919fa77c670b7ed5c284bcb66770ea71fc");
}

#[test]
fn version_reads_dependencies() {
    let version: ModrinthVersion = fixture("version-with-dependencies");
    let dependency = &version.dependencies[0];

    assert_eq!(dependency.dependency_type, ModrinthDependencyType::Required);
    assert_eq!(dependency.project_id.as_ref().unwrap().0, "AANobbMI");
    assert_eq!(dependency.file_name, None);
}

#[test]
fn an_unrecognised_enum_value_reads_as_unknown() {
    fn read<T: serde::de::DeserializeOwned>(value: &str) -> T {
        serde_json::from_str(&format!("\"{value}\"")).expect(value)
    }

    assert_eq!(
        read::<ModrinthSideSupport>("client_and_server"),
        ModrinthSideSupport::Unknown
    );
    assert_eq!(
        read::<ModrinthDependencyType>("conflicts_with"),
        ModrinthDependencyType::Unknown
    );
    assert_eq!(
        read::<ModrinthVersionType>("nightly"),
        ModrinthVersionType::Unknown
    );
}

#[test]
fn query_encodes_arrays_as_json() {
    let query = ModrinthVersionQuery {
        loaders: vec!["paper".to_owned()],
        game_versions: vec!["1.21.1".to_owned()],
        featured: None,
    };

    assert_eq!(
        query.parameters(),
        [
            ("loaders", r#"["paper"]"#.to_owned()),
            ("game_versions", r#"["1.21.1"]"#.to_owned()),
        ]
    );
}

#[test]
fn empty_query_sends_no_parameters() {
    assert_eq!(ModrinthVersionQuery::default().parameters(), []);
}

#[test]
fn a_release_resolves_to_its_primary_file_and_checksums() {
    let versions: Vec<ModrinthVersion> = fixture("versions");
    let addon = ModrinthAddon {
        id: "luckperms".into(),
        version: None,
        files: Vec::new(),
    };

    let version = select_version(&versions, None).expect("a release");
    let files = select_files(version, &addon.files).expect("the primary file");
    let resolved = to_resolved(&addon, version, &files);

    assert_eq!(resolved.resolved.version.as_deref(), Some("v5.5.71-bukkit"));
    assert_eq!(resolved.resolved.files, ["LuckPerms-Bukkit-5.5.71.jar"]);

    let PackageSource::Download(download) = &resolved.package.sources[0] else {
        panic!("a modrinth file is a download");
    };
    assert_eq!(download.size, Some(1501521));
    assert!(!download.checksums.is_empty());
    assert_eq!(
        resolved.package.artifacts[0].destination().to_str(),
        Some("LuckPerms-Bukkit-5.5.71.jar")
    );
}

#[test]
fn an_exact_version_that_does_not_exist_matches_nothing() {
    let versions: Vec<ModrinthVersion> = fixture("versions");

    assert!(select_version(&versions, Some("9.9.9")).is_none());
    assert!(select_files(&versions[0], &["missing.jar".to_owned()]).is_err());
}

#[test]
fn paper_asks_for_every_loader_a_paper_server_runs() {
    let platform = Platform::Paper(PaperPlatform {
        minecraft: Some("1.21.1".into()),
        build: None,
    });

    assert_eq!(loaders_for(&platform), ["paper", "spigot", "bukkit"]);
}
