# Eidra Agent Guide

Eidra is a local-first trust layer for AI development environments.

## Product intent

- Inspect what AI tools are about to send off-device.
- Decide how that traffic should be handled with local policy.
- Prove what happened later through audit logs and explicit runtime diagnostics.

The goal is not to block everything. The goal is to make `observe -> protect -> enforce` practical in real AI workflows.

## Core commands

- `cargo run -q -p eidra-core -- doctor`
- `cargo run -q -p eidra-core -- setup shell`
- `cargo run -q -p eidra-core -- scan --json`
- `cargo run -q -p eidra-core -- config validate --json`
- `cargo run -q -p eidra-core -- start`
- `cargo run -q -p eidra-core -- gateway`

## Repo map

- `crates/eidra-core`: CLI entrypoints, runtime config loading, setup/doctor/config UX.
- `crates/eidra-proxy`: HTTP/HTTPS proxy, policy enforcement, local route behavior.
- `crates/eidra-router`: request transformation and local LLM routing helpers.
- `crates/eidra-policy`: YAML policy types and evaluation engine.
- `crates/eidra-scan`: built-in and custom secret/PII scanning rules.
- `crates/eidra-mcp`: MCP firewall, semantic tool validation, rate limiting.
- `crates/eidra-audit`: local SQLite-backed audit logging.
- `crates/eidra-tui`: local terminal dashboard.

## Important current behaviors

- `route(local)` is currently designed for OpenAI-compatible chat completion requests.
- `proxy.max_body_size` is enforced before scanning/policy execution after request body collection.
- `setup --write` writes generated artifacts under `~/.eidra/generated/<target>` instead of editing user files directly.
- `scan --json`, `doctor --json`, and `config validate --json` are intended for CI and other automation.

## Good next upgrades

- Add streaming-aware body limiting instead of post-collection checks.
- Expand local routing beyond OpenAI-compatible chat completions.
- Add `setup --write` output formats for more IDE- and MCP-native config files.
- Keep localized READMEs aligned with `README.md` as commands evolve.
