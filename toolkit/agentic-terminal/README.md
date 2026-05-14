# agentic-terminal

Foundation TUI for the Agentic Terminal OS: fuzzy tool navigation (ratatui + nucleo) with a pluggable agent harness, background runtime, and event journal.

## Run

```bash
cargo run --manifest-path toolkit/agentic-terminal/Cargo.toml
```

The UI lists default CLI tools with live fuzzy search (`/ opens the command palette`, `Esc` closes it).
