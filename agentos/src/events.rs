//! Append-only audit trail (memory-only for now). Supports future Merkle/event hashing.

use crate::model::{InputMode, ToolId};

/// Audit events — payloads kept for future hashing / export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AppEvent {
    ModeChanged {
        from: InputMode,
        to: InputMode,
    },
    LayerChanged {
        from: &'static str,
        to: &'static str,
    },
    QueryChanged(String),
    SelectionChanged(Option<ToolId>),
    AgentLineSubmitted(String),
    /// Worker returned a result for a stale filter generation.
    StaleWorkerResultDropped {
        epoch: u64,
        current_epoch: u64,
    },
    /// Worker channel was full; job dropped (should be rare with bounded cap).
    FilterDispatchBackpressured,
    Quit,
}

pub struct EventLog {
    entries: Vec<AppEvent>,
    max: usize,
}

impl EventLog {
    pub fn new(max: usize) -> Self {
        Self {
            entries: Vec::new(),
            max,
        }
    }

    pub fn push(&mut self, e: AppEvent) {
        self.entries.push(e);
        let overflow = self.entries.len().saturating_sub(self.max);
        if overflow > 0 {
            self.entries.drain(0..overflow);
        }
    }

    pub fn tail(&self, n: usize) -> &[AppEvent] {
        let len = self.entries.len();
        let start = len.saturating_sub(n);
        &self.entries[start..]
    }
}
