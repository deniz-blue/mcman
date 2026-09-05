use std::path::{Path, PathBuf};

use kdl::KdlNode;
use miette::{IntoDiagnostic, Result};

use crate::core::kdl::{Errors, Reader};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ExecuteTask {
    pub command: String,
    pub directory: Option<PathBuf>,
}

impl ExecuteTask {
    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut reader = Reader::new(node, errors);
        let command = reader.required_argument("command");
        let directory = reader.path_property("cd");
        reader.reject_unread();

        Self { command, directory }
    }

    pub async fn run(&self, working_dir: &Path) -> Result<()> {
        let current_dir = working_dir.join(self.directory.clone().unwrap_or_default());

        let mut child = tokio::process::Command::new(&self.command)
            .current_dir(&current_dir)
            .spawn()
            .into_diagnostic()?;

        // TODO: global console outputting

        child.wait().await.into_diagnostic()?;

        Ok(())
    }
}
