//! Append-only journaled events.

use crate::types::JournalEvent;

pub trait Journal: Send + Sync {
    fn append(&mut self, event: JournalEvent);

    fn iter(&self) -> Box<dyn Iterator<Item = &JournalEvent> + '_>;
}

pub struct InMemoryJournal {
    events: Vec<JournalEvent>,
}

impl InMemoryJournal {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for InMemoryJournal {
    fn default() -> Self {
        Self::new()
    }
}

impl Journal for InMemoryJournal {
    fn append(&mut self, event: JournalEvent) {
        self.events.push(event);
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &JournalEvent> + '_> {
        Box::new(self.events.iter())
    }
}
