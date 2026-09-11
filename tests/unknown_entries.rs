use kdl::{KdlDocument, KdlNode};
use mcman::{lockfile::Lockfile, manifest::Manifest};
use std::fs;

#[derive(Clone, Copy)]
enum Parser {
    Manifest,
    Lockfile,
}

impl Parser {
    fn accepts(self, name: &str, text: &str) -> bool {
        match self {
            Self::Manifest => Manifest::parse(name, text).is_ok(),
            Self::Lockfile => Lockfile::parse(name, text).is_ok(),
        }
    }

    fn wildcard_nodes(self) -> &'static [&'static str] {
        match self {
            Self::Manifest => &[],
            Self::Lockfile => &["platform"],
        }
    }

    fn accepting(name: &str, text: &str) -> Option<Self> {
        [Self::Manifest, Self::Lockfile]
            .into_iter()
            .find(|parser| parser.accepts(name, text))
    }
}

fn node_count(nodes: &[KdlNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            1 + node
                .children()
                .map(|kids| node_count(kids.nodes()))
                .unwrap_or(0)
        })
        .sum()
}

fn inject_unknown_property(
    nodes: &mut [KdlNode],
    target: usize,
    seen: &mut usize,
) -> Option<String> {
    for node in nodes {
        if *seen == target {
            node.push(("zzz", 1));
            return Some(node.name().value().to_owned());
        }
        *seen += 1;

        if let Some(children) = node.children_mut().as_mut() {
            let hit = inject_unknown_property(children.nodes_mut(), target, seen);
            if hit.is_some() {
                return hit;
            }
        }
    }

    None
}

#[test]
fn every_node_rejects_a_property_it_does_not_know() {
    let mut checked = 0;

    for entry in fs::read_dir("tests/fixtures").expect("fixtures directory") {
        let path = entry.expect("directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("kdl") {
            continue;
        }

        let name = path.to_string_lossy().into_owned();
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}"));

        let Some(parser) = Parser::accepting(&name, &text) else {
            panic!("{name} is not read by either parser, so injection proves nothing");
        };

        let document: KdlDocument = text.parse().unwrap_or_else(|e| panic!("{name}: {e}"));

        for target in 0..node_count(document.nodes()) {
            let mut injected: KdlDocument = text.parse().expect("the fixture parsed once already");
            let hit = inject_unknown_property(injected.nodes_mut(), target, &mut 0)
                .expect("target is in range");

            if parser.wildcard_nodes().contains(&hit.as_str()) {
                continue;
            }

            assert!(
                !parser.accepts(&name, &injected.to_string()),
                "{name}: `{hit}` accepted an unknown property `zzz`",
            );
            checked += 1;
        }
    }

    assert!(
        checked > 150,
        "only {checked} nodes were injected into — the corpus is not being walked"
    );
}
