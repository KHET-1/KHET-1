use crate::model::{Tool, ToolId};

pub fn builtin_tools() -> Vec<Tool> {
    const DATA: &[(&str, &str)] = &[
        ("git", "content tracker — clone, commit, branch, merge"),
        ("cargo", "Rust package manager — build, test, doc"),
        ("rustc", "Rust compiler"),
        ("rg", "ripgrep — fast recursive grep"),
        ("fd", "user-friendly find alternative"),
        ("bat", "syntax-highlighting cat"),
        ("jq", "JSON processor"),
        ("nix", "Nix package manager — reproducible builds"),
        ("nixos-rebuild", "switch NixOS system configuration"),
        ("ssh", "OpenSSH remote shell"),
        ("curl", "transfer URLs"),
        ("tar", "tape archive utility"),
        ("systemctl", "systemd service control"),
        ("journalctl", "systemd journal query"),
        ("docker", "container runtime CLI"),
        ("kubectl", "Kubernetes control plane CLI"),
    ];

    DATA.iter()
        .enumerate()
        .map(|(i, (name, desc))| Tool {
            id: i as ToolId,
            name: (*name).into(),
            description: (*desc).into(),
        })
        .collect()
}
