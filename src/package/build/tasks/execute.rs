use std::path::PathBuf;

use knus::Decode;
use miette::{IntoDiagnostic, Result};

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ExecuteTask {
    #[knus(argument)]
    pub command: String,
    #[knus(property(name = "cd"))]
    pub directory: Option<PathBuf>,
}

impl ExecuteTask {
    pub async fn run(&self, working_dir: &PathBuf) -> Result<()> {
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
