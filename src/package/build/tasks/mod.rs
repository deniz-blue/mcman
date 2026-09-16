use std::path::PathBuf;

use kdl::KdlNode;

use crate::{
    core::kdl::{Errors, KdlRead, KdlWrite},
    package::build::tasks::execute::ExecuteTask,
};

pub mod execute;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildTask {
    Execute(ExecuteTask),
}

impl BuildTask {
    pub(crate) fn read(node: &KdlNode, errors: &mut Errors) -> Self {
        Self::Execute(ExecuteTask::read_node(node, errors))
    }

    pub fn node_name(&self) -> &'static str {
        match self {
            Self::Execute(_) => "execute",
        }
    }
}

impl KdlWrite for BuildTask {
    fn write(&self, node: &mut KdlNode) {
        match self {
            Self::Execute(execute) => execute.write(node),
        }
    }
}

pub struct BuildTaskContext {
    pub path: PathBuf,
}

pub trait BuildTaskRunner {
    async fn run(&self, context: &BuildTaskContext) -> miette::Result<()>;
}
