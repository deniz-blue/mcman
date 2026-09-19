use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Sender},
    Arc,
};
use std::thread::JoinHandle;

use crate::output::terminal;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TaskId(pub usize);

pub enum Progress {
    Note(String),
    Bytes { done: u64, total: Option<u64> },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    Muted,
    Accent,
    Failure,
}

#[derive(Clone, Copy)]
pub struct Tag {
    pub text: &'static str,
    pub emphasis: Emphasis,
}

impl Tag {
    pub const fn new(text: &'static str, emphasis: Emphasis) -> Self {
        Self { text, emphasis }
    }
}

pub enum Event {
    Create {
        id: TaskId,
        parent: Option<TaskId>,
        name: String,
    },
    Finish {
        id: TaskId,
        tag: Tag,
        text: String,
        detail: String,
    },
    Progress {
        id: TaskId,
        progress: Progress,
    },
    Line {
        id: TaskId,
        text: String,
    },
    Abandon {
        id: TaskId,
    },
    Suspend,
    Resume,
    Close,
}

struct Channel {
    events: Sender<Event>,
    next_id: AtomicUsize,
}

impl Channel {
    fn send(&self, event: Event) {
        let _ = self.events.send(event);
    }
}

pub struct TaskReporter {
    channel: Arc<Channel>,
    thread: Option<JoinHandle<()>>,
}

impl TaskReporter {
    pub fn open() -> Self {
        let (events, incoming) = mpsc::channel();

        Self {
            channel: Arc::new(Channel {
                events,
                next_id: AtomicUsize::new(0),
            }),
            thread: Some(std::thread::spawn(move || terminal::run(incoming))),
        }
    }

    pub fn task(&self, name: impl Into<String>) -> Task {
        Task::open(&self.channel, None, name.into())
    }

    pub fn suspend(&self) -> Suspended<'_> {
        self.channel.send(Event::Suspend);

        Suspended {
            channel: &self.channel,
        }
    }
}

impl Drop for TaskReporter {
    fn drop(&mut self) {
        self.channel.send(Event::Close);

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct Suspended<'a> {
    channel: &'a Channel,
}

impl Drop for Suspended<'_> {
    fn drop(&mut self) {
        self.channel.send(Event::Resume);
    }
}

pub struct Task {
    channel: Arc<Channel>,
    id: TaskId,
    finished: bool,
}

impl Task {
    fn open(channel: &Arc<Channel>, parent: Option<TaskId>, name: String) -> Task {
        let id = TaskId(channel.next_id.fetch_add(1, Ordering::Relaxed));
        channel.send(Event::Create { id, parent, name });

        Task {
            channel: Arc::clone(channel),
            id,
            finished: false,
        }
    }

    pub fn task(&self, name: impl Into<String>) -> Task {
        Task::open(&self.channel, Some(self.id), name.into())
    }

    pub fn set_progress(&self, progress: Progress) {
        self.channel.send(Event::Progress {
            id: self.id,
            progress,
        });
    }

    pub fn line(&self, text: impl Into<String>) {
        self.channel.send(Event::Line {
            id: self.id,
            text: text.into(),
        });
    }

    pub fn finish(mut self, tag: Tag, text: impl Into<String>, detail: impl Into<String>) {
        self.channel.send(Event::Finish {
            id: self.id,
            tag,
            text: text.into(),
            detail: detail.into(),
        });
        self.finished = true;
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        if !self.finished {
            self.channel.send(Event::Abandon { id: self.id });
        }
    }
}
