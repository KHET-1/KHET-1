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
            name: "nix".into(),
            description: "Nix package manager and language".into(),
            examples: vec![
                "nix flake init".into(),
                "nix develop".into(),
                "nix build".into(),
                "nix search nixpkgs".into(),
            ],
            package: Some("nix".into()),
            nix_attr: Some("nix".into()),
            version: None,
            homepage: Some("https://nixos.org".into()),
        },
        Tool {
            name: "nix-shell".into(),
            description: "Enter a shell with packages from Nix (legacy)".into(),
            examples: vec![
                "nix-shell -p python3".into(),
                "nix-shell -p gcc make".into(),
                "nix-shell < shell.nix".into(),
            ],
            package: Some("nix".into()),
            nix_attr: Some("nix".into()),
            version: None,
            homepage: Some("https://nixos.org".into()),
        },
        Tool {
            name: "nixos-rebuild".into(),
            description: "Rebuild and switch to new NixOS configuration".into(),
            examples: vec![
                "sudo nixos-rebuild switch".into(),
                "sudo nixos-rebuild test".into(),
                "sudo nixos-rebuild dry-build".into(),
            ],
            package: Some("nixos-tools".into()),
            nix_attr: Some("nixos-rebuild".into()),
            version: None,
            homepage: Some("https://nixos.org".into()),
        },
        Tool {
            name: "journalctl".into(),
            description: "Query and display systemd journal logs".into(),
            examples: vec![
                "journalctl -u service-name".into(),
                "journalctl -f".into(),
                "journalctl -p err -b".into(),
                "journalctl --since '1 hour ago'".into(),
            ],
            package: Some("systemd".into()),
            nix_attr: None,
            version: None,
            homepage: Some("https://www.freedesktop.org/software/systemd/man/journalctl.html".into()),
        },
        Tool {
            name: "dmesg".into(),
            description: "Print or control kernel ring buffer (boot/firmware messages)".into(),
            examples: vec![
                "dmesg | tail -20".into(),
                "dmesg | grep -i error".into(),
                "dmesg | grep -i firmware".into(),
                "sudo dmesg -c".into(),
            ],
            package: Some("util-linux".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "nvme".into(),
            description: "NVMe drive discovery, diagnostics & firmware management".into(),
            examples: vec![
                "nvme list".into(),
                "nvme list-ns /dev/nvme0n1".into(),
                "nvme id-ctrl /dev/nvme0".into(),
                "nvme smart-log /dev/nvme0".into(),
                "sudo nvme fw-commit /dev/nvme0 -s <slot>".into(),
            ],
            package: Some("nvme-cli".into()),
            nix_attr: Some("nvme-cli".into()),
            version: None,
            homepage: Some("https://github.com/linux-nvme/nvme-cli".into()),
        },
        Tool {
            name: "lsblk".into(),
            description: "List block devices (disks, partitions, NVMe)".into(),
            examples: vec![
                "lsblk".into(),
                "lsblk -f".into(),
                "lsblk -S".into(),
                "lsblk -I 259".into(),
            ],
            package: Some("util-linux".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "smartctl".into(),
            description: "Monitor & diagnose storage device health (S.M.A.R.T.)".into(),
            examples: vec![
                "sudo smartctl -a /dev/nvme0n1".into(),
                "sudo smartctl -H /dev/sda".into(),
                "sudo smartctl -l error /dev/sda".into(),
                "sudo smartctl --scan".into(),
            ],
            package: Some("smartmontools".into()),
            nix_attr: Some("smartmontools".into()),
            version: None,
            homepage: Some("https://www.smartmontools.org".into()),
        },
        Tool {
            name: "efibootmgr".into(),
            description: "View and modify UEFI boot entries & NVMe boot order".into(),
            examples: vec![
                "efibootmgr".into(),
                "sudo efibootmgr -c -L 'NixOS' -l /vmlinuz".into(),
                "sudo efibootmgr -o 0,1,2 -n 0".into(),
                "sudo efibootmgr -B -b 0".into(),
            ],
            package: Some("efibootmgr".into()),
            nix_attr: Some("efibootmgr".into()),
            version: None,
            homepage: None,
        },
        Tool {
            name: "systemd-boot".into(),
            description: "UEFI boot manager configuration & debugging".into(),
            examples: vec![
                "bootctl status".into(),
                "sudo bootctl update".into(),
                "sudo bootctl install".into(),
                "bootctl list".into(),
            ],
            package: Some("systemd".into()),
            nix_attr: None,
            version: None,
            homepage: Some("https://www.freedesktop.org/wiki/Software/systemd/EFI/".into()),
        },
        Tool {
            name: "systemctl".into(),
            description: "Control systemd services (start, stop, status, enable)".into(),
            examples: vec![
                "systemctl status".into(),
                "sudo systemctl restart service-name".into(),
                "systemctl list-units --failed".into(),
                "systemctl list-units --type=device".into(),
            ],
            package: Some("systemd".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
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
        Tool {
            name: "git".into(),
            description: "Version control system".into(),
            examples: vec![
                "git clone <repo>".into(),
                "git add -A && git commit -m 'message'".into(),
                "git log --oneline".into(),
            ],
            package: Some("git".into()),
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
