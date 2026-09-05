use std::path::PathBuf;

use knus::Decode;

use crate::package::build::tasks::execute::ExecuteTask;

pub mod execute;

#[derive(Decode, Clone, Debug, PartialEq, Eq, Hash)]
#[knus(span_type = knus::span::Span)]
pub enum BuildTask {
    Execute(ExecuteTask),
}

pub struct BuildTaskContext {
    pub path: PathBuf,
}

pub trait BuildTaskRunner {
    async fn run(&self, context: &BuildTaskContext) -> miette::Result<()>;
}
