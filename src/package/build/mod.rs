use knus::Decode;

use crate::package::build::tasks::{BuildTask, BuildTaskRunner};

pub mod tasks;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PackageBuild {
    #[knus(children)]
    pub tasks: Vec<BuildTask>,
}

impl BuildTaskRunner for PackageBuild {
    async fn run(
        &self,
        context: &crate::package::build::tasks::BuildTaskContext,
    ) -> miette::Result<()> {
        for task in &self.tasks {
            match task {
                BuildTask::Execute(execute_task) => {
                    execute_task.run(&context.path).await?;
                }
            }
        }
        Ok(())
    }
}
