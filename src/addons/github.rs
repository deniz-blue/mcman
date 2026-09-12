use std::fmt::Display;

use kdl::KdlNode;

use crate::{addons::AddonType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitHubAddon {
    pub repository: String,
    pub tag: Option<String>,
    pub asset: Option<String>,
}

impl AddonType for GitHubAddon {
    const TYPE_NAME: &'static str = "github";


    fn read(reader: &mut Reader) -> Self {
        Self {
            repository: reader.required_argument_or_property("repository"),
            tag: reader.property("tag"),
            asset: reader.property("asset"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        node.push(self.repository.as_str());
        if let Some(tag) = &self.tag {
            node.push(("tag", tag.as_str()));
        }
        if let Some(asset) = &self.asset {
            node.push(("asset", asset.as_str()));
        }
    }
}

impl Display for GitHubAddon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", Self::TYPE_NAME, self.repository)
    }
}
