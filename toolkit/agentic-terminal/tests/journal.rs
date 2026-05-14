use agentic_terminal::journal::{InMemoryJournal, Journal};
use agentic_terminal::types::{AgentId, JournalEvent};
use std::time::SystemTime;

#[test]
fn append_increments_len() {
    let mut j = InMemoryJournal::new();
    assert_eq!(j.len(), 0);
    j.append(JournalEvent::AgentInvoked {
        agent: AgentId("help".into()),
        args: vec![],
        at: SystemTime::UNIX_EPOCH,
    });
    assert_eq!(j.len(), 1);
}

#[test]
fn iter_preserves_insertion_order() {
    let mut j = InMemoryJournal::new();
    j.append(JournalEvent::AgentResult {
        agent: AgentId("a".into()),
        ok: true,
        summary: "1".into(),
        at: SystemTime::UNIX_EPOCH,
    });
    j.append(JournalEvent::AgentResult {
        agent: AgentId("b".into()),
        ok: false,
        summary: "2".into(),
        at: SystemTime::UNIX_EPOCH,
    });
    let summaries: Vec<_> = j
        .iter()
        .filter_map(|e| match e {
            JournalEvent::AgentResult { summary, .. } => Some(summary.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(summaries, vec!["1", "2"]);
}
