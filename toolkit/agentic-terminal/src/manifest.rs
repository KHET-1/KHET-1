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

#[allow(clippy::too_many_lines)]
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
            name: "nmcli".into(),
            description: "NetworkManager CLI for WiFi/SSID management".into(),
            examples: vec![
                "nmcli dev wifi list".into(),
                "nmcli con show".into(),
                "nmcli dev wifi connect SSID password PASSWORD".into(),
                "nmcli con modify SSID wifi.powersave off".into(),
            ],
            package: Some("networkmanager".into()),
            nix_attr: Some("networkmanager".into()),
            version: None,
            homepage: Some("https://networkmanager.dev/".into()),
        },
        Tool {
            name: "iwconfig".into(),
            description: "Configure WiFi interface (legacy, for diagnostics)".into(),
            examples: vec![
                "iwconfig".into(),
                "iwconfig wlan0".into(),
                "iwconfig wlan0 mode Managed".into(),
                "iwconfig wlan0 txpower 20".into(),
            ],
            package: Some("wireless-tools".into()),
            nix_attr: Some("wireless-tools".into()),
            version: None,
            homepage: None,
        },
        Tool {
            name: "ip".into(),
            description: "Configure network interfaces, routing, and connectivity".into(),
            examples: vec![
                "ip addr show".into(),
                "ip link show".into(),
                "ip route show".into(),
                "sudo ip addr add 192.168.1.100/24 dev eth0".into(),
            ],
            package: Some("iproute2".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "ping".into(),
            description: "Test network connectivity to hosts".into(),
            examples: vec![
                "ping -c 4 8.8.8.8".into(),
                "ping -4 example.com".into(),
                "ping -6 example.com".into(),
                "ping -W 2 192.168.1.1".into(),
            ],
            package: Some("iputils".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "mount".into(),
            description: "Mount/unmount filesystems and connected drives".into(),
            examples: vec![
                "mount".into(),
                "sudo mount /dev/sda1 /mnt".into(),
                "sudo umount /mnt".into(),
                "sudo mount -o remount,rw /".into(),
            ],
            package: Some("util-linux".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "diff".into(),
            description: "Compare files and directories line by line".into(),
            examples: vec![
                "diff file1 file2".into(),
                "diff -u old.txt new.txt".into(),
                "diff -r dir1 dir2".into(),
                "diff --color file1 file2".into(),
            ],
            package: Some("diffutils".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "meld".into(),
            description: "Visual diff and merge tool for files and directories".into(),
            examples: vec![
                "meld file1 file2".into(),
                "meld dir1 dir2".into(),
                "meld --help".into(),
            ],
            package: Some("meld".into()),
            nix_attr: Some("meld".into()),
            version: None,
            homepage: Some("http://meldmerge.org/".into()),
        },
        Tool {
            name: "nano".into(),
            description: "Simple text editor for quick edits".into(),
            examples: vec![
                "nano file.txt".into(),
                "nano +10 file.txt".into(),
                "nano -w file.txt".into(),
            ],
            package: Some("nano".into()),
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "rsync".into(),
            description: "Sync files and swap/transfer drives efficiently".into(),
            examples: vec![
                "rsync -av source/ dest/".into(),
                "rsync -av --delete source/ dest/".into(),
                "rsync -avz user@remote:/path /local/path".into(),
                "sudo rsync -av /mnt/old/ /mnt/new/".into(),
            ],
            package: Some("rsync".into()),
            nix_attr: None,
            version: None,
            homepage: Some("https://rsync.samba.org/".into()),
        },
        Tool {
            name: "dd".into(),
            description: "Low-level drive copy/backup/restore (handle with care!)".into(),
            examples: vec![
                "sudo dd if=/dev/sda of=/backup/sda.img".into(),
                "sudo dd if=/backup/sda.img of=/dev/sdb".into(),
                "sudo dd if=/dev/zero of=/dev/sda bs=1M".into(),
                "sudo dd if=/dev/urandom of=/dev/sda bs=4M status=progress".into(),
            ],
            package: Some("coreutils".into()),
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
