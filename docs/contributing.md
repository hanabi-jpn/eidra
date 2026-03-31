# Contributing to Eidra

Thank you for your interest in contributing to Eidra!

## Getting Started

```bash
git clone https://github.com/hanabi-jpn/eidra.git
cd eidra
cargo build
cargo test
```

## Development

- Rust edition 2021
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Test: `cargo test`

## Adding Scan Rules

New scan rules go in `crates/eidra-scan/src/rules/builtin.rs`. Each rule needs:
1. A `Rule` struct with name, regex pattern, category, severity, and description
2. At least 2 tests (match + non-match)

## Project Structure

```
crates/
  eidra-core/      CLI binary
  eidra-scan/      Data classification engine
  eidra-policy/    Policy evaluation
  eidra-proxy/     HTTP/HTTPS intercept proxy
  eidra-router/    LLM routing + masking
  eidra-audit/     Local SQLite audit log
  eidra-tui/       Terminal UI dashboard
  eidra-mcp/       MCP gateway
  eidra-transport/ E2EE communication
  eidra-identity/  Device-bound identity
  eidra-seal/      Sealed metadata
```

## Code Standards

- `thiserror` in libraries, `anyhow` in binary
- No `unwrap()` in library code
- All async: `tokio`
- Logging: `tracing`
- All config structs: `#[derive(Serialize, Deserialize)]`

## License

MIT
