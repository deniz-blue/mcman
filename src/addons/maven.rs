use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlVariant, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MavenAddon {
    pub group: String,
    pub artifact: String,
    pub repository: Option<String>,
    pub version: Option<String>,
    pub classifier: Option<String>,
    pub extension: Option<String>,
}

impl KdlVariant for MavenAddon {
    const TYPE_NAME: &'static str = "maven";
}

impl KdlRead for MavenAddon {
    fn read(reader: &mut Reader) -> Self {
        Self {
            group: reader.required_argument_or_property("group"),
            artifact: reader.required_argument_or_property("artifact"),
            repository: reader.property("repository"),
            version: reader.property("version"),
            classifier: reader.property("classifier"),
            extension: reader.property("extension"),
        }
    }
}

impl KdlWrite for MavenAddon {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.group.as_str());
        node.push(self.artifact.as_str());
        if let Some(repository) = &self.repository {
            node.push(("repository", repository.as_str()));
        }
        if let Some(version) = &self.version {
            node.push(("version", version.as_str()));
        }
        if let Some(classifier) = &self.classifier {
            node.push(("classifier", classifier.as_str()));
        }
        if let Some(extension) = &self.extension {
            node.push(("extension", extension.as_str()));
        }
    }
}
