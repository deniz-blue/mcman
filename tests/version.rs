use mcman::addons::{RequestedVersion, Stability};

fn requested(text: &str) -> RequestedVersion {
    RequestedVersion::from(text.to_owned())
}

#[test]
fn the_magic_words_are_the_only_ones() {
    assert_eq!(requested("latest"), RequestedVersion::Latest);
    assert_eq!(requested("beta"), RequestedVersion::Beta);
    assert_eq!(requested("alpha"), RequestedVersion::Alpha);
    assert_eq!(
        requested("1.10"),
        RequestedVersion::Exact("1.10".to_owned())
    );
    assert_eq!(
        requested("v5.5.71-bukkit"),
        RequestedVersion::Exact("v5.5.71-bukkit".to_owned())
    );
}

#[test]
fn latest_takes_only_releases() {
    let latest = requested("latest");

    assert!(latest.matches("1.10", Stability::Release));
    assert!(!latest.matches("1.10", Stability::Beta));
    assert!(!latest.matches("1.10", Stability::Alpha));
}

#[test]
fn beta_takes_a_release_too() {
    let beta = requested("beta");

    assert!(beta.matches("1.10", Stability::Release));
    assert!(beta.matches("1.10", Stability::Beta));
    assert!(!beta.matches("1.10", Stability::Alpha));
}

#[test]
fn alpha_takes_anything() {
    let alpha = requested("alpha");

    for stability in [Stability::Release, Stability::Beta, Stability::Alpha] {
        assert!(alpha.matches("1.10", stability), "{stability:?}");
    }
}

#[test]
fn an_exact_version_ignores_stability() {
    let exact = requested("1.10");

    assert!(exact.matches("1.10", Stability::Alpha));
    assert!(!exact.matches("1.11", Stability::Release));
}

#[test]
fn a_version_named_after_a_magic_word_is_unreachable() {
    assert_eq!(requested("beta"), RequestedVersion::Beta);
    assert!(requested("beta").matches("anything", Stability::Release));
}

#[test]
fn writing_and_reparsing_returns_the_same_request() {
    for text in ["latest", "beta", "alpha", "1.10", "v5.5.71-bukkit"] {
        let parsed = requested(text);

        assert_eq!(parsed.to_string(), text);
        assert_eq!(requested(&parsed.to_string()), parsed);
    }
}
