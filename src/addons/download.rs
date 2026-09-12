use kdl::KdlNode;

use crate::{addons::AddonType, core::kdl::Reader, package::source::download::Download};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadAddon(pub Download);

impl AddonType for DownloadAddon {
    const TYPE_NAME: &'static str = "download";

    fn read(reader: &mut Reader) -> Self {
        Self(Download::read_from(reader))
    }

    fn write(&self, node: &mut KdlNode) {
        self.0.write(node);
    }
}
