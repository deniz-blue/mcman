use std::path::Path;

use miette::Result;

use crate::{ctx::Context, task::{Task, TaskHandle}};

pub struct UnzipTask<'a> {
    pub zip_path: &'a Path,
    pub destination: &'a Path,
}

impl<'a> Task for UnzipTask<'a> {
    type Output = ();

    async fn run(&self, _ctx: &Context, _handle: &TaskHandle) -> Result<Self::Output> {
        todo!()
    }
}
