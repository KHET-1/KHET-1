//! Cross-cutting boundary types (manifest, hashing, verification).

use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AgentId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn of(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    pub fn hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for b in &self.0 {
            use std::fmt::Write as _;
            let _ = write!(s, "{b:02x}");
        }
        s
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Signature(pub Vec<u8>);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PublicKey(pub Vec<u8>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolSource {
    BuiltinDefault,
    Path(PathBuf),
    NixpkgsRev(String),
    Url(String),
}

#[derive(Clone, Debug)]
pub struct ToolManifest {
    pub source: ToolSource,
    pub content_hash: Hash,
    pub created_at: SystemTime,
    pub tools: Vec<Tool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub examples: Vec<String>,
    pub package: Option<String>,
    pub nix_attr: Option<String>,
    pub version: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootChainLink {
    pub stage: String,
    pub measurement: Hash,
    pub verified: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRef {
    pub kind: String,
    pub locator: String,
}

#[derive(Clone, Debug)]
pub struct VerificationReport {
    pub root: Hash,
    pub chain: Vec<BootChainLink>,
    pub evidence: Vec<EvidenceRef>,
    pub verified: bool,
    pub verified_at: SystemTime,
    pub signer: AgentId,
    pub signature: Signature,
}

#[derive(Clone, Debug)]
pub enum JournalEvent {
    AgentInvoked {
        agent: AgentId,
        args: Vec<String>,
        at: SystemTime,
    },
    AgentResult {
        agent: AgentId,
        ok: bool,
        summary: String,
        at: SystemTime,
    },
    ManifestLoaded {
        source: ToolSource,
        content_hash: Hash,
        at: SystemTime,
    },
    Verification(VerificationReport),
}

#[cfg(test)]
mod tests {
    use super::Hash;

    #[test]
    fn hash_of_is_deterministic() {
        let h1 = Hash::of(b"hello");
        let h2 = Hash::of(b"hello");
        assert_eq!(h1.0, h2.0);
    }

    #[test]
    fn hash_hex_is_64_chars() {
        let h = Hash::of(b"x");
        assert_eq!(h.hex().len(), 64);
    }
}
