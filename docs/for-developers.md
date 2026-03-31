# Eidra For Developers

Eidra is a local-first proxy and MCP firewall for AI development workflows.

If you are building with Cursor, Claude Code, Codex CLI, OpenAI or Anthropic SDK apps, agent runtimes, or MCP servers, Eidra gives you a programmable trust boundary in front of those systems.

Named setup targets are available today for Cursor, Claude Code, Codex CLI, OpenAI-compatible SDKs, Anthropic-compatible SDKs, GitHub Actions, and MCP.

## What you get

- request scanning for secrets, PII, and risky content
- YAML policy evaluation for allow, mask, block, and route decisions
- local routing of sensitive OpenAI-compatible chat requests to Ollama
- local audit logs and a live TUI
- MCP gateway controls for server allowlists, tool rules, response scanning, and rate limiting
- machine-readable output for CI and automation

## Integration Paths

### CLI and local tools

Use the proxy and dashboard when you want human visibility into live traffic.

```bash
eidra dashboard
```

### SDK and script workflows

Use the machine-readable outputs to gate or inspect flows in automation.

```bash
eidra doctor --json
eidra scan --json
eidra config validate --json
```

### MCP workflows

Use the gateway when you want policy between AI clients and tool servers.

```bash
eidra gateway
```

## Setup Targets

Eidra ships guidance for common environments:

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup codex
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

Use `--write` to generate reusable setup artifacts without editing your local files directly.

```bash
eidra setup cursor --write
```

## Common Developer Workflows

### 1. Scan only

Use this when you want low-friction validation in CI, pre-commit, or experiments.

```bash
eidra scan path/to/file
```

### 2. Proxy plus dashboard

Use this when you want to see live requests, findings, and actions.

```bash
eidra dashboard
```

### 3. MCP gateway

Use this when you want tool-level policy for agent workflows.

```bash
eidra gateway
```

## Why developers adopt it

- it works with the tools they already use
- it is local-first
- it is easy to inspect
- it is scriptable
- it is modular enough to grow from solo use to team workflows

## Recommended Reading

- [What Is Eidra?](what-is-eidra.md)
- [Architecture](architecture.md)
- [Media Kit](media-kit.md)
