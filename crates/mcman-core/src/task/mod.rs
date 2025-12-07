use miette::Result;

use crate::ctx::Context;

mod download_task;
mod executor;
pub use download_task::*;
mod checksum_task;
pub use checksum_task::*;
mod copy_task;
pub use copy_task::*;
mod execute_task;
pub use execute_task::*;
mod unzip_task;
pub use unzip_task::*;

pub trait Task: Send + Sync {
    type Output: Send + 'static;
    async fn run(&self, ctx: &Context, handle: &TaskHandle) -> Result<Self::Output>;
}

pub struct TaskHandle {}
impl TaskHandle {
    pub fn update_progress(&self, _progress: f32) {
        todo!()
    }

    pub fn set_message(&self, _message: &str) {
        todo!()
    }
}
