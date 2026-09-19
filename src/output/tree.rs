use std::collections::VecDeque;
use std::time::Instant;

use indexmap::IndexMap;

use crate::output::task::{Progress, TaskId};

const OUTPUT_KEEP: usize = 20;

pub struct Entry {
    pub parent: Option<TaskId>,
    pub name: String,
    pub progress: Option<Progress>,
    pub output: VecDeque<String>,
    pub started: Instant,
    pub done: bool,
}

#[derive(Default)]
pub struct Tree {
    entries: IndexMap<TaskId, Entry>,
}

impl Tree {
    pub fn create(&mut self, id: TaskId, parent: Option<TaskId>, name: String) {
        self.entries.insert(
            id,
            Entry {
                parent,
                name,
                progress: None,
                output: VecDeque::new(),
                started: Instant::now(),
                done: false,
            },
        );
    }

    pub fn set_progress(&mut self, id: TaskId, progress: Progress) {
        if let Some(entry) = self.at(id) {
            entry.progress = Some(progress);
        }
    }

    pub fn line(&mut self, id: TaskId, text: String) {
        let Some(entry) = self.at(id) else { return };

        entry.output.push_back(text);
        while entry.output.len() > OUTPUT_KEEP {
            entry.output.pop_front();
        }
    }

    pub fn finish(&mut self, id: TaskId) -> Option<(String, VecDeque<String>)> {
        let entry = self.at(id)?;

        entry.done = true;

        Some((entry.name.clone(), std::mem::take(&mut entry.output)))
    }

    pub fn abandon(&mut self, id: TaskId) {
        if let Some(entry) = self.at(id) {
            entry.done = true;
            entry.output.clear();
        }
    }

    pub fn children(&self, parent: Option<TaskId>) -> impl Iterator<Item = (TaskId, &Entry)> {
        self.entries
            .iter()
            .filter_map(move |(&id, entry)| (entry.parent == parent).then_some((id, entry)))
    }

    pub fn counts(&self, id: TaskId) -> Option<(usize, usize)> {
        let mut total = 0;
        let mut done = 0;

        for (_, entry) in self.children(Some(id)) {
            total += 1;
            done += usize::from(entry.done);
        }

        (total > 0).then_some((done, total))
    }

    fn at(&mut self, id: TaskId) -> Option<&mut Entry> {
        self.entries.get_mut(&id)
    }
}
