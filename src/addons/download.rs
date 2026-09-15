use kdl::KdlNode;

use crate::{
    core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader},
    package::source::download::Download,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadAddon(pub Download);

impl KdlVariant for DownloadAddon {
    const TYPE_NAME: &'static str = "download";
}

impl KdlRead for DownloadAddon {
    fn read(reader: &mut Reader) -> Self {
        Self(KdlRead::read(reader))
    }
}

impl KdlWrite for DownloadAddon {
    fn write(&self, node: &mut KdlNode) {
        self.0.write(node);
    }
}
