# agentic-terminal

Foundation TUI for the Agentic Terminal OS: fuzzy tool navigation (ratatui + nucleo) with a pluggable agent harness, background runtime, and event journal.

## Features

- **Fuzzy search** over 26 built-in commands and tools
- **Helper-only mode** for NixOS: navigate commands without agent execution
- **Session memory**: remembers frequently used and pinned commands
- **System diagnostics**: NVMe boot, firmware, journal, S.M.A.R.T. health checks
- **Networking**: WiFi/SSID discovery, connectivity testing (ping), network config
- **Drive management**: Connect/mount/sync drives, file diffs, secure wipe, backups
- **Nix/NixOS integration**: commands for development, configuration rebuilds, and package management
- **Live search**: Real-time matching with nucleo fuzzy matcher

## Built-in Commands (26 tools)

### Nix/NixOS
- `nix` — Package manager and Nix language
- `nix-shell` — Enter a shell with dev dependencies
- `nixos-rebuild` — Rebuild and switch NixOS config

### System Diagnostics & Forensics
- `journalctl` — Query systemd journal and error logs
- `dmesg` — Kernel ring buffer (boot/firmware messages)
- `nvme` — NVMe drive discovery and diagnostics
- `lsblk` — List block devices and partitions
- `smartctl` — S.M.A.R.T. health monitoring
- `efibootmgr` — UEFI boot entry management
- `systemd-boot` — Boot manager configuration
- `systemctl` — Control systemd services

### Networking & WiFi
- `nmcli` — NetworkManager CLI for WiFi/SSID management and connection profiles
- `iwconfig` — Configure WiFi interface (legacy diagnostics)
- `ip` — Configure network interfaces, routing, and connectivity
- `ping` — Test network connectivity to hosts and gateways

### Drive & File Management
- `mount` — Mount/unmount filesystems and connected drives
- `diff` — Compare files and directories line by line
- `meld` — Visual diff and merge tool for files and directories
- `nano` — Simple text editor for quick edits
- `rsync` — Sync and swap files between drives efficiently
- `dd` — Low-level drive copy/backup/restore (handle with care!)

### Utilities
- `ripgrep` — Ultra-fast text search
- `fd` — Fast file finder
- `bat` — Syntax-highlighted cat
- `jq` — JSON processor
- `git` — Version control

## Run

### Default mode (with agent)
```bash
cargo run --manifest-path toolkit/agentic-terminal/Cargo.toml
```

### Helper-only mode (NixOS, no agent)
```bash
cargo run --manifest-path toolkit/agentic-terminal/Cargo.toml -- --helper-only
```

In helper-only mode:
- Use `/` to open the search palette (read-only, agent commands disabled)
- Use arrow keys, Page Up/Down to navigate tools
- Press `Enter` to remember/mark a command
- View command examples, package names, and descriptions
- All navigation works offline without agent dispatch

## Controls

| Key | Action |
|-----|--------|
| `q` | Quit |
| `/` | Open command palette (toggle read-only in helper mode) |
| `↑` `↓` | Navigate tools |
| `PgUp` `PgDn` | Page up/down |
| Wheel/Click | Mouse navigation |
| `Enter` | Open tool (agent) / Remember (helper-only) |
| `Esc` | Close palette |

## Testing

```bash
cargo test
cargo clippy --all-targets
```

All tests pass, including:
- Helper mode configuration
- Session state and command memory
- Navigator fuzzy matching with 26 tools
- Tool catalog integrity

## Use Cases

### NixOS System Forensics
Use helper-only mode to navigate common diagnostic commands when checking for:
- **Boot issues**: `efibootmgr`, `systemd-boot`, `dmesg`
- **NVMe firmware**: `nvme`, `smartctl`, `lsblk`
- **System health**: `journalctl -p err`, `systemctl list-units --failed`
- **Configuration**: `nixos-rebuild dry-build` to validate changes

Example workflow on NixOS:
```bash
# Start helper in NixOS shell
nix-shell -p agentic-terminal --run "agentic-terminal --helper-only"

# Search for diagnostics:
# Type: "nvme" → see all NVMe commands
# Type: "journal" → see journalctl examples
# Type: "smart" → see S.M.A.R.T. health commands
```

### Network Connectivity Checks
- **WiFi discovery**: `nmcli dev wifi list` → find available SSID networks
- **Connection status**: `nmcli con show` → see all active profiles
- **Test connectivity**: `ping -c 4 8.8.8.8` → verify internet
- **View interfaces**: `ip addr show` → check IP configuration

Example workflow:
```bash
agentic-terminal --helper-only
# Search: "wifi" → see nmcli examples
# Search: "ping" → see connectivity test commands
# Search: "ip" → see network config commands
```

### Drive Management & Swapping
- **List drives**: `lsblk` → see connected storage
- **Sync drives**: `rsync -av /source/ /dest/` → sync with verification
- **Full backup**: `sudo dd if=/dev/sda of=/backup.img` → backup entire drive
- **Compare files**: `meld file1 file2` → visual diff before swap
- **Edit configs**: `nano /etc/config` → quick configuration edits

Example workflow:
```bash
# Search: "mount" → mount new drive
# Search: "rsync" → sync files
# Search: "diff" → compare before swap
# Search: "dd" → backup/restore
```

## Architecture

- **lib.rs** — Core modules (navigation, state, agent harness)
- **helper_mode.rs** — Configuration for agent-free operation
- **manifest.rs** — Default tool catalog (Nix + diagnostics + networking + drives)
- **state.rs** — Session state: recent/pinned commands
- **app.rs** — Main TUI loop, event routing
- **view/** — SearchView (navigation) and CommandPaletteView (input)
- **navigator.rs** — Nucleo-backed fuzzy matching
- **runtime.rs** — Tokio worker thread and agent dispatch

## Notes

- MSRV: Rust 1.75
- No unsafe code
- Clippy: all + pedantic (with explicit allows for large tool catalog)
- All tests pass (39 tests: 7 lib unit + 32 integration tests)
- Tool catalog: 26 commands covering Nix, diagnostics, networking, and drive management

