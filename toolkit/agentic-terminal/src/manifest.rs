//! Tool manifest construction and hashing.

use crate::types::{Hash, Tool, ToolManifest, ToolSource};

fn push_len(out: &mut Vec<u8>, len: usize) {
    let len_u64 = u64::try_from(len).expect("length exceeds u64");
    out.extend_from_slice(&len_u64.to_le_bytes());
}

fn push_field(out: &mut Vec<u8>, label: &[u8], value: &[u8]) {
    push_len(out, label.len());
    out.extend_from_slice(label);
    push_len(out, value.len());
    out.extend_from_slice(value);
}

/// Deterministic serialization of tools for hashing and verification.
pub fn canonical_tool_bytes(tools: &[Tool]) -> Vec<u8> {
    let mut buf = Vec::new();
    push_len(&mut buf, tools.len());

    for tool in tools {
        push_field(&mut buf, b"name:", tool.name.as_bytes());
        push_field(&mut buf, b"desc:", tool.description.as_bytes());
        push_len(&mut buf, tool.examples.len());
        for example in &tool.examples {
            push_field(&mut buf, b"example:", example.as_bytes());
        }
        let pkg = tool.package.clone().unwrap_or_default();
        push_field(&mut buf, b"package:", pkg.as_bytes());
        let nix_attr = tool.nix_attr.clone().unwrap_or_default();
        push_field(&mut buf, b"nix_attr:", nix_attr.as_bytes());
        let version = tool.version.clone().unwrap_or_default();
        push_field(&mut buf, b"version:", version.as_bytes());
        let homepage = tool.homepage.clone().unwrap_or_default();
        push_field(&mut buf, b"homepage:", homepage.as_bytes());
    }
    buf
}

pub fn default_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "ripgrep".into(),
            description: "Ultra-fast text search tool".into(),
            examples: vec!["rg pattern .".into(), "rg -i pattern".into()],
            package: Some("ripgrep".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "fd".into(),
            description: "Fast alternative to find".into(),
            examples: vec!["fd pattern".into(), "fd -e rs".into()],
            package: Some("fd".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "bat".into(),
            description: "Cat clone with syntax highlighting".into(),
            examples: vec!["bat Cargo.toml".into(), "bat -n src/main.rs".into()],
            package: Some("bat".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "jq".into(),
            description: "Command-line JSON processor".into(),
            examples: vec![
                "jq '.items[]' file.json".into(),
                "cat x.json | jq '.a.b'".into(),
            ],
            package: Some("jq".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
    ]
}

pub fn default_manifest() -> ToolManifest {
    let tools = default_tools();
    let bytes = canonical_tool_bytes(&tools);
    ToolManifest {
        source: ToolSource::BuiltinDefault,
        content_hash: Hash::of(&bytes),
        created_at: std::time::SystemTime::now(),
        tools,
    }
}
