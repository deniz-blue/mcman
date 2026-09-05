use knus::Decode;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
#[knus(span_type = knus::span::Span)]
pub enum DependencyId {
    Modrinth(ModrinthProject),
}

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[knus(span_type = knus::span::Span)]
pub struct ModrinthProject {
    pub id: String,
    pub version: Option<String>,
}
