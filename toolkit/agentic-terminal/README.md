# agentic-terminal

Foundation TUI for the Agentic Terminal OS: fuzzy tool navigation (ratatui + nucleo) with a pluggable agent harness, background runtime, and event journal.

## Features

- **Fuzzy search** over 16+ built-in commands and tools
- **Helper-only mode** for NixOS: navigate commands without agent execution
- **Session memory**: remembers frequently used and pinned commands
- **System diagnostics**: NVMe boot, firmware, journal, S.M.A.R.T. health checks
- **Nix/NixOS integration**: commands for development, configuration rebuilds, and package management
- **Live search**: Real-time matching with nucleo fuzzy matcher

## Built-in Commands

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
- Navigator fuzzy matching with 16+ tools
- Tool catalog integrity

## Use Case: NixOS System Forensics

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
# Press Enter to remember frequently used commands
```

## Architecture

- **lib.rs** — Core modules (navigation, state, agent harness)
- **helper_mode.rs** — Configuration for agent-free operation
- **manifest.rs** — Default tool catalog (Nix + diagnostics)
- **state.rs** — Session state: recent/pinned commands
- **app.rs** — Main TUI loop, event routing
- **view/** — SearchView (navigation) and CommandPaletteView (input)
- **navigator.rs** — Nucleo-backed fuzzy matching
- **runtime.rs** — Tokio worker thread and agent dispatch

## Notes

- MSRV: Rust 1.75
- No unsafe code
- Clippy: all + pedantic
- All tests pass (15+ unit/integration tests)
