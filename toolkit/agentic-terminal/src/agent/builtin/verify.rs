//! Verification stub emitting a structured report.

use std::time::SystemTime;

use crate::agent::{Agent, AgentCtx, AgentEvent, AgentOutcome};
use crate::types::{EvidenceRef, Hash, Signature, VerificationReport};

#[derive(Clone, Copy, Debug, Default)]
pub struct VerifyAgent;

fn merkle_root_for_chain(chain: &[crate::types::BootChainLink]) -> Hash {
    let mut acc = Vec::new();
    for link in chain {
        acc.extend_from_slice(link.stage.as_bytes());
        acc.extend_from_slice(&link.measurement.0);
        acc.push(u8::from(link.verified));
    }
    if acc.is_empty() {
        return Hash::of(b"nucleus:verification:stub:empty_chain");
    }
    Hash::of(&acc)
}

impl Agent for VerifyAgent {
    fn name(&self) -> &'static str {
        "verify"
    }

    fn help(&self) -> &'static str {
        "Produce a structured (stub) boot-chain verification report."
    }

    fn run(&mut self, _args: &[String], ctx: &mut AgentCtx<'_>) -> AgentOutcome {
        let chain = Vec::<crate::types::BootChainLink>::new();
        let evidence = Vec::<EvidenceRef>::new();
        let root = merkle_root_for_chain(&chain);
        let signer = self.identity();
        let report = VerificationReport {
            root,
            chain,
            evidence,
            verified: false,
            verified_at: SystemTime::now(),
            signer: signer.clone(),
            signature: Signature::default(),
        };
        let summary = format!(
            "stub report root={} verified=false chain_len=0",
            report.root.hex()
        );
        ctx.responder
            .send(AgentEvent::Verification(report.clone()));
        ctx.responder.send(AgentEvent::Status {
            agent: signer,
            text: summary,
        });
        AgentOutcome::Ok
    }
}
