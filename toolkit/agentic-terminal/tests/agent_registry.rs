use agentic_terminal::agent::{Agent, AgentCtx, AgentId, AgentOutcome, AgentRegistry};
use agentic_terminal::journal::InMemoryJournal;
use agentic_terminal::manifest::default_manifest;
use crossbeam_channel::bounded;

#[derive(Default)]
struct AlphaAgent;

impl Agent for AlphaAgent {
    fn name(&self) -> &'static str {
        "alpha"
    }

    fn help(&self) -> &'static str {
        "alpha test"
    }

    fn identity(&self) -> AgentId {
        AgentId("alpha".into())
    }

    fn run(&mut self, _args: &[String], _ctx: &mut AgentCtx<'_>) -> AgentOutcome {
        AgentOutcome::Ok
    }
}

#[derive(Default)]
struct BetaAgent;

impl Agent for BetaAgent {
    fn name(&self) -> &'static str {
        "beta"
    }

    fn help(&self) -> &'static str {
        "beta test"
    }

    fn identity(&self) -> AgentId {
        AgentId("beta".into())
    }

    fn run(&mut self, _args: &[String], _ctx: &mut AgentCtx<'_>) -> AgentOutcome {
        AgentOutcome::Ok
    }
}

#[test]
fn registry_roundtrip_and_unknown() {
    let mut reg = AgentRegistry::new();
    reg.register(AlphaAgent);
    reg.register(BetaAgent);

    let mut journal = InMemoryJournal::new();
    let mut manifest = default_manifest();
    let (sender, _rx) = bounded(4);
    let responder = agentic_terminal::agent::AgentResponder::new(sender);
    let mut ctx = AgentCtx {
        responder,
        journal: &mut journal,
        manifest: &mut manifest,
    };

    assert!(reg.dispatch("alpha", &mut ctx).is_ok());
    assert!(reg.dispatch("beta", &mut ctx).is_ok());
    assert!(reg.dispatch("gamma", &mut ctx).is_err());
}

#[test]
fn names_sorted_and_identities_distinct() {
    let mut reg = AgentRegistry::new();
    reg.register(AlphaAgent);
    reg.register(BetaAgent);

    assert_eq!(reg.names(), vec!["alpha", "beta"]);

    let a = AlphaAgent.identity();
    let b = BetaAgent.identity();
    assert_ne!(a, b);
}
