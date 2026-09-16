use std::path::{Path, PathBuf};

use miette::{bail, IntoDiagnostic, Result};

use kdl::KdlNode;

use crate::core::kdl::{KdlRead, KdlWrite, Reader};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ExecuteTask {
    pub program: String,
    pub arguments: Vec<String>,
    pub directory: Option<PathBuf>,
}

impl KdlRead for ExecuteTask {
    fn read(reader: &mut Reader) -> Self {
        let command = reader.required_argument("command");
        let directory = reader.path_property("cd");

        let words = match shell_words::split(&command) {
            Ok(words) => words,
            Err(_) => {
                reader.reject(format!("`{command}` has an unbalanced quote"));
                Vec::new()
            }
        };

        let mut words = words.into_iter();
        let program = words.next().unwrap_or_default();
        let arguments: Vec<String> = words.collect();

        if program.is_empty() {
            reader.reject("`execute` needs a command to run".to_owned());
        }

        Self {
            program,
            arguments,
            directory,
        }
    }
}

impl KdlWrite for ExecuteTask {
    fn write(&self, node: &mut KdlNode) {
        let command = shell_words::join(std::iter::once(&self.program).chain(&self.arguments));
        node.push(command);

        if let Some(directory) = &self.directory {
            node.push(("cd", directory.display().to_string()));
        }
    }
}

impl ExecuteTask {
    pub async fn run(&self, working_dir: &Path) -> Result<()> {
        let current_dir = working_dir.join(self.directory.clone().unwrap_or_default());

        // TODO: global console outputting

        let status = tokio::process::Command::new(&self.program)
            .args(&self.arguments)
            .current_dir(&current_dir)
            .spawn()
            .into_diagnostic()?
            .wait()
            .await
            .into_diagnostic()?;

        if !status.success() {
            let command = shell_words::join(std::iter::once(&self.program).chain(&self.arguments));
            bail!("`{command}` exited with {status}");
        }

        Ok(())
    }
}
