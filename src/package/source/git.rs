use std::path::{Path, PathBuf};

use kdl::KdlNode;
use miette::{bail, IntoDiagnostic, Result};

use crate::core::kdl::{KdlRead, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Git {
    pub url: String,
    pub path: Option<PathBuf>,
}

impl KdlRead for Git {
    fn read(reader: &mut Reader) -> Self {
        Self {
            url: reader.required_argument("repository url"),
            path: reader.path_property("path"),
        }
    }
}

impl KdlWrite for Git {
    fn write(&self, node: &mut KdlNode) {
        node.push(self.url.as_str());
        if let Some(path) = &self.path {
            node.push(("path", path.display().to_string()));
        }
    }
}

impl Git {
    pub fn destination(&self) -> &Path {
        match &self.path {
            Some(path) => path,
            None => Path::new(""),
        }
    }

    pub async fn run(&self, workspace: &Path) -> Result<String> {
        let checkout = workspace.join(self.destination());

        let status = tokio::process::Command::new("git")
            .args(["clone", "--depth", "1", &self.url])
            .arg(&checkout)
            .status()
            .await
            .into_diagnostic()?;

        if !status.success() {
            bail!("cloning `{}` exited with {status}", self.url);
        }

        let output = tokio::process::Command::new("git")
            .arg("-C")
            .arg(&checkout)
            .args(["rev-parse", "HEAD"])
            .output()
            .await
            .into_diagnostic()?;

        if !output.status.success() {
            bail!("`{}` has no commit to read", self.url);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
}
