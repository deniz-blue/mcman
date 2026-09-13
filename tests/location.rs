use std::path::Path;

use mcman::core::location::Location;

fn location(text: &str) -> Location {
    Location::from(text.to_owned())
}

#[test]
fn a_relative_reference_resolves_beside_its_base() {
    let manifest = location("/srv/pack/mcman.kdl");
    let joined = manifest.join("packs/base.mrpack").expect("joins");
    assert_eq!(
        joined,
        Location::Path(Path::new("/srv/pack/packs/base.mrpack").to_owned())
    );

    let pack = location("https://example.com/pack/pack.toml");
    let joined = pack.join("index.toml").expect("joins");
    assert_eq!(joined.to_string(), "https://example.com/pack/index.toml");
}

#[test]
fn an_absolute_reference_replaces_its_base() {
    let manifest = location("/srv/pack/mcman.kdl");

    let url = manifest
        .join("https://example.com/Pack.mrpack")
        .expect("joins");
    assert_eq!(url.to_string(), "https://example.com/Pack.mrpack");

    let path = manifest.join("/tmp/Pack.mrpack").expect("joins");
    assert_eq!(
        path,
        Location::Path(Path::new("/tmp/Pack.mrpack").to_owned())
    );
}
