use agentic_terminal::agent::builtin::register_builtins;
use agentic_terminal::agent::{AgentCtx, AgentOutcome, AgentRegistry, AgentResponder};
use agentic_terminal::journal::InMemoryJournal;
use agentic_terminal::manifest::default_manifest;
use crossbeam_channel::bounded;

fn exec(line: &str) -> Result<AgentOutcome, String> {
    let mut registry = AgentRegistry::new();
    register_builtins(&mut registry);
    let mut journal = InMemoryJournal::new();
    let mut manifest = default_manifest();
    let (sender, _receiver) = bounded(8);
    let responder = AgentResponder::new(sender);
    let mut ctx = AgentCtx {
        responder,
        journal: &mut journal,
        manifest: &mut manifest,
    };
    registry.dispatch(line, &mut ctx)
}

#[test]
fn dispatches_refresh() {
    assert!(matches!(exec("refresh").unwrap(), AgentOutcome::Ok));
}

#[test]
fn dispatches_verify() {
    assert!(matches!(exec("verify").unwrap(), AgentOutcome::Ok));
}

#[test]
fn dispatches_search_with_args() {
    assert!(matches!(exec("search foo bar").unwrap(), AgentOutcome::Ok));
}

#[test]
fn dispatches_help_aliases() {
    assert!(matches!(exec("help").unwrap(), AgentOutcome::Ok));
    assert!(matches!(exec("?").unwrap(), AgentOutcome::Ok));
}

#[test]
fn dispatches_quit_variants() {
    assert!(matches!(exec("quit").unwrap(), AgentOutcome::Shutdown));
    assert!(matches!(exec("exit").unwrap(), AgentOutcome::Shutdown));
}

#[test]
fn rejects_empty_or_whitespace() {
    assert!(exec("").unwrap_err().contains("Empty"));
    assert!(exec("    ").unwrap_err().contains("Empty"));
}

#[test]
fn rejects_unknown_commands() {
    assert!(exec("nope").unwrap_err().contains("Unknown"));
}

#[test]
fn trims_before_dispatch() {
    assert!(matches!(exec("  refresh  ").unwrap(), AgentOutcome::Ok));
}

#[test]
fn ignores_command_case_for_refresh() {
    assert!(matches!(exec("REFRESH").unwrap(), AgentOutcome::Ok));
}

#[test]
fn search_requires_arguments() {
    match exec("search") {
        Ok(AgentOutcome::Error(msg)) => assert!(msg.contains("Usage")),
        other => panic!("unexpected outcome: {other:?}"),
    }
}
