use kdl::KdlNode;

use crate::{addons::AddonType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModrinthAddon {
    pub id: String,
    pub version: Option<String>,
    pub files: Vec<String>,
}

impl AddonType for ModrinthAddon {
    const TYPE_NAME: &'static str = "modrinth";

    fn read(reader: &mut Reader) -> Self {
        Self {
            id: reader.required_argument_or_property("id"),
            version: reader.property("version"),
            files: reader.list_property("files"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        node.push(self.id.as_str());
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
        if !self.files.is_empty() {
            node.push(("files", self.files.join(" ")));
        }
    }
}
