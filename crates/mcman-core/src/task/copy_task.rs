use std::path::{Path, PathBuf};

use miette::Result;

use crate::{ctx::Context, task::{Task, TaskHandle}};

pub struct CopyTask<'a> {
    pub source: &'a Path,
    pub destination: &'a Path,
}

impl<'a> Task for CopyTask<'a> {
    type Output = PathBuf;

    async fn run(&self, _ctx: &Context, _handle: &TaskHandle) -> Result<Self::Output> {
        todo!()
    }
}

