mod block;
mod format;
mod row;
mod spinner;
mod task;
mod terminal;
mod tree;

pub use task::{Emphasis, Progress, Tag, Task, TaskReporter};

pub const UNCHANGED: Tag = Tag::new("Unchanged", Emphasis::Muted);
pub const RESOLVED: Tag = Tag::new("Resolved", Emphasis::Accent);
pub const READ: Tag = Tag::new("Read", Emphasis::Accent);
pub const BUILT: Tag = Tag::new("Built", Emphasis::Accent);
pub const DOWNLOADED: Tag = Tag::new("Downloaded", Emphasis::Accent);
pub const PLACED: Tag = Tag::new("Placed", Emphasis::Accent);
pub const FAILED: Tag = Tag::new("Failed", Emphasis::Failure);
