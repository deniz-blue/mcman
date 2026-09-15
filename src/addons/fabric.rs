use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricLoaderAddon {
    pub loader: Option<String>,
    pub installer: Option<String>,
}

impl KdlVariant for FabricLoaderAddon {
    const TYPE_NAME: &'static str = "fabric";
}

impl KdlRead for FabricLoaderAddon {
    fn read(reader: &mut Reader) -> Self {
        Self {
            loader: reader.property("loader"),
            installer: reader.property("installer"),
        }
    }
}

impl KdlWrite for FabricLoaderAddon {
    fn write(&self, node: &mut KdlNode) {
        if let Some(loader) = &self.loader {
            node.push(("loader", loader.as_str()));
        }
        if let Some(installer) = &self.installer {
            node.push(("installer", installer.as_str()));
        }
    }
}
