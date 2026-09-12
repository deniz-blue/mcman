use std::fmt::Display;

use kdl::KdlNode;

use crate::{addons::AddonType, core::kdl::Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricLoaderAddon {
    pub loader: Option<String>,
    pub installer: Option<String>,
}

impl AddonType for FabricLoaderAddon {
    const TYPE_NAME: &'static str = "fabric";


    fn read(reader: &mut Reader) -> Self {
        Self {
            loader: reader.property("loader"),
            installer: reader.property("installer"),
        }
    }

    fn write(&self, node: &mut KdlNode) {
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
        if let Some(installer) = &self.installer {
            node.push(("installer", installer.as_str()));
        }
    }
}

impl Display for FabricLoaderAddon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::TYPE_NAME)
    }
}
