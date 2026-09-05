use std::path::PathBuf;

use kdl::KdlNode;

use crate::{core::kdl::Errors, package::build::tasks::execute::ExecuteTask};

pub mod execute;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildTask {
    Execute(ExecuteTask),
}

impl BuildTask {
    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        Self::Execute(ExecuteTask::read(node, errors))
    }
}

pub struct BuildTaskContext {
    pub path: PathBuf,
}

pub trait BuildTaskRunner {
    async fn run(&self, context: &BuildTaskContext) -> miette::Result<()>;
}
