//! Trust and persistence **boundary types** — stubs with stable shapes so
//! crypto, Merkle manifests, and remote `StateStore` backends can land later
//! without rewiring every callsite.
//!
//! Some items are only exercised from tests today; they still pin the public
//! surface area for manifests, signatures, and storage backends.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crate::model::{Tool, ToolId};

/// Content-addressed blob id (e.g. SHA-256 / BLAKE3). All-zero = “unset” stub.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    pub const PLACEHOLDER: Self = Self([0; 32]);
}

/// Opaque detached signature bytes (e.g. minisign / OpenSSL) — not parsed yet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Signature(pub Vec<u8>);

/// Stable agent identity for manifests and journal correlation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AgentId(pub Arc<str>);

impl AgentId {
    pub fn new(name: &str) -> Self {
        Self(Arc::from(name))
    }
}

/// Declarative description of a tool row as it would appear in a sealed manifest.
#[derive(Clone, Debug)]
pub struct ToolManifest {
    pub tool_id: ToolId,
    pub name: Arc<str>,
    pub content_hash: ContentHash,
    /// Reserved for lattice / Merkle linkage once manifests are sealed.
    pub merkle_root_hint: Option<ContentHash>,
}

pub fn tool_manifest_stub(tool: &Tool) -> ToolManifest {
    ToolManifest {
        tool_id: tool.id,
        name: Arc::from(tool.name.as_str()),
        content_hash: ContentHash::PLACEHOLDER,
        merkle_root_hint: None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationReport {
    Skipped { reason: &'static str },
    Passed { tool: ToolId },
    Failed { tool: ToolId, detail: String },
}

pub fn verify_tool_stub(manifest: &ToolManifest) -> VerificationReport {
    if manifest.content_hash == ContentHash::PLACEHOLDER {
        VerificationReport::Skipped {
            reason: "placeholder content hash (wire real hashing next)",
        }
    } else {
        VerificationReport::Passed {
            tool: manifest.tool_id,
        }
    }
}

/// Key-value persistence boundary (local dir, SQLite, Nextcloud WebDAV, …).
pub trait StateStore: Send {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn put(&mut self, key: &str, value: &[u8]) -> io::Result<()>;
    fn delete(&mut self, key: &str) -> io::Result<()>;
}

#[derive(Debug, Default)]
pub struct MemoryStateStore {
    inner: HashMap<String, Vec<u8>>,
}

impl MemoryStateStore {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl StateStore for MemoryStateStore {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.get(key).cloned()
    }

    fn put(&mut self, key: &str, value: &[u8]) -> io::Result<()> {
        self.inner.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&mut self, key: &str) -> io::Result<()> {
        self.inner.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_stub_skips_placeholder_hash() {
        let t = Tool {
            id: 0,
            name: "git".into(),
            description: "x".into(),
        };
        let m = tool_manifest_stub(&t);
        assert!(matches!(
            verify_tool_stub(&m),
            VerificationReport::Skipped { .. }
        ));
    }

    #[test]
    fn memory_state_store_roundtrip() {
        let mut s = MemoryStateStore::new();
        s.put("k", b"v").unwrap();
        assert_eq!(s.get("k").unwrap(), b"v");
        s.delete("k").unwrap();
        assert!(s.get("k").is_none());
    }

    #[test]
    fn state_store_via_trait_object() {
        let mut s: Box<dyn StateStore> = Box::new(MemoryStateStore::new());
        s.put("a", b"1").unwrap();
        assert_eq!(s.get("a").unwrap(), b"1");
        s.delete("a").unwrap();
        assert!(s.get("a").is_none());
    }

    #[test]
    fn signature_and_failed_report_constructible() {
        let _sig = Signature(vec![1, 2, 3]);
        let fail = VerificationReport::Failed {
            tool: 0,
            detail: "demo".into(),
        };
        assert!(matches!(fail, VerificationReport::Failed { .. }));
    }
}
