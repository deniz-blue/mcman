use kdl::KdlNode;

use crate::{
    core::kdl::{child_nodes, reject_node, Errors},
    package::build::tasks::{BuildTask, BuildTaskRunner},
};

pub mod tasks;

const BUILD_NODES: &str = "execute";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PackageBuild {
    pub tasks: Vec<BuildTask>,
}

impl PackageBuild {
    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        let mut build = Self::default();

        for child in child_nodes(node) {
            match child.name().value() {
                "execute" => build.tasks.push(BuildTask::read(child, errors)),
                _ => reject_node(child, errors, BUILD_NODES),
            }
        }

        build
    }
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
