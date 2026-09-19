use crate::output::{
    task::TaskId,
    tree::{Entry, Tree},
};

const BLOCK_ROWS: usize = 13;
const OUTPUT_SHOW: usize = 4;

pub enum Slot<'a> {
    Blank,
    Entry {
        entry: &'a Entry,
        depth: usize,
        counts: Option<(usize, usize)>,
    },
    Output {
        text: &'a str,
        depth: usize,
    },
}

pub fn slots(tree: &Tree) -> Vec<Slot<'_>> {
    let mut slots = Vec::new();
    collect(tree, None, 1, &mut slots);

    if slots.is_empty() {
        return slots;
    }

    slots.insert(0, Slot::Blank);
    slots.truncate(BLOCK_ROWS);

    slots
}

fn collect<'a>(tree: &'a Tree, parent: Option<TaskId>, depth: usize, slots: &mut Vec<Slot<'a>>) {
    for (id, entry) in tree.children(parent) {
        let counts = tree.counts(id);

        if entry.done || (entry.progress.is_none() && entry.output.is_empty() && counts.is_none()) {
            continue;
        }

        slots.push(Slot::Entry {
            entry,
            depth,
            counts,
        });

        let tail = entry.output.len().saturating_sub(OUTPUT_SHOW);
        slots.extend(
            entry
                .output
                .iter()
                .skip(tail)
                .map(|text| Slot::Output { text, depth }),
        );

        collect(tree, Some(id), depth + 1, slots);
    }
}
