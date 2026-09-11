use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestedVersion {
    Latest,
    Beta,
    Alpha,
    Exact(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stability {
    Alpha,
    Beta,
    Release,
}

impl RequestedVersion {
    pub fn matches(&self, version: &str, stability: Stability) -> bool {
        match self {
            Self::Latest => stability >= Stability::Release,
            Self::Beta => stability >= Stability::Beta,
            Self::Alpha => true,
            Self::Exact(wanted) => wanted == version,
        }
    }
}

impl From<String> for RequestedVersion {
    fn from(text: String) -> Self {
        match text.as_str() {
            "latest" => Self::Latest,
            "beta" => Self::Beta,
            "alpha" => Self::Alpha,
            _ => Self::Exact(text),
        }
    }
}

impl Display for RequestedVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Latest => "latest",
            Self::Beta => "beta",
            Self::Alpha => "alpha",
            Self::Exact(version) => version,
        })
    }
}
