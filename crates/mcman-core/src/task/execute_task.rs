use std::path::Path;

use miette::Result;

use crate::{ctx::Context, task::{Task, TaskHandle}};

pub struct ExecuteTask<'a> {
    pub program: &'a str,
    pub args: &'a [String],
    pub dir: Option<&'a Path>,
}

impl<'a> Task for ExecuteTask<'a> {
    type Output = ();

    async fn run(&self, _ctx: &Context, _handle: &TaskHandle) -> Result<Self::Output> {
        // TODO: use tokio::process::Command to run program
        todo!()
    }
}
