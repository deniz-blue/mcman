use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DependencyId {
    Modrinth(ModrinthProject),
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ModrinthProject {
    pub id: String,
    pub version: Option<String>,
}
