# Hacker News — Show HN Post

## Title (max 80 chars)

```
Show HN: Eidra – Zero-trust firewall for AI agents and MCP tools (Rust, OSS)
```

### Alternative titles:
```
Show HN: Eidra – Your AI agent just ran DROP TABLE. We stop that. (Rust, OSS)
Show HN: Eidra – Semantic RBAC for MCP: block "rm -rf" but allow "ls" (Rust)
```

## Comment (post this as first comment)

```
Hi HN, I built Eidra because I got tired of blindly trusting AI tools with my code.

The problem: Every time you use Cursor, Claude Code, or Copilot, your entire file context — including .env files, API keys, database credentials — gets sent to cloud APIs. Recent research shows Copilot repos leak secrets 40% more often [1], Claude Code silently reads .env files [2], and MCP tools have CVSS 8.6+ vulnerabilities [3].

Eidra is a local proxy (Rust, ~5ms overhead) that sits between you and your AI tools:

- Scans every AI request for 47 types of secrets (AWS keys, tokens, PII, private keys, Japanese phone numbers, etc.)
- Masks or blocks before data leaves your machine
- Routes sensitive requests to local LLMs (Ollama) instead of cloud
- MCP firewall with tool-level access control
- Real-time TUI dashboard so you can see everything
- E2EE channels (X25519 + ChaCha20) for when AI can't help

The scan engine has 47 regex rules with 97 tests. The policy engine uses YAML (block private keys, mask API keys, allow PII for local LLM). Everything runs on-device — Eidra never phones home.

Trust model: Content is E2EE (we can't read it). Metadata uses split-key encryption (Eidra alone can't decrypt). Everything is in the open-source code.

Technical choices:
- Rust for performance (<5ms proxy overhead target)
- hyper 1.x for HTTP, rustls for TLS, ratatui for TUI
- HTTPS interception via local CA (MITM only for AI provider domains)
- 11 modular crates — use the scan engine alone, or the full proxy

The architecture is inspired by GoodCreate's @POP technology — "the next entity that knows you best is your own device."

What I'd love feedback on:
1. Which scan rules are you missing?
2. Should Eidra integrate directly into IDE extensions instead of being a proxy?
3. How do you handle AI security at your company?

[1] https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage
[2] https://devops.com/security-flaws-in-anthropics-claude-code-risk-stolen-data-system-takeover/
[3] https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html
```

## Alternative Titles (A/B test)

1. `Show HN: Eidra – Your AI is leaking your secrets. See it happen in real-time`
2. `Show HN: Eidra – A proxy that scans and masks secrets before they reach AI APIs`
3. `Show HN: Eidra – MCP firewall + AI data flow scanner (Rust, 47 rules, TUI)`

## Timing

Best HN posting times:
- Tuesday-Thursday, 9-11am EST (6-8am PST)
- Avoid weekends and Mondays

## Subreddit Posts

### r/rust
```
Title: Eidra: Edge-native AI security proxy in Rust — 11 crates, 47 scan rules, ratatui TUI

Built a proxy that intercepts AI requests and scans for secrets before they leave your machine. Rust + hyper 1.x + rustls + ratatui. Would love code review from the community.

https://github.com/hanabi-jpn/eidra
```

### r/programming
```
Title: Claude Code reads your .env without asking. Copilot leaks secrets 40% more. I built a fix.

Eidra is an open-source proxy that sits between you and your AI tools. It scans every request for 47 types of secrets, masks or blocks before data leaves your machine, and gives you a real-time dashboard of everything flowing.

https://github.com/hanabi-jpn/eidra
```

### r/MachineLearning
```
Title: [P] Eidra: Open-source AI data flow scanner + MCP firewall

Scans AI API requests for secrets/PII, masks before sending, routes sensitive requests to local LLMs. Also includes an MCP gateway with tool-level access control. 47 scan rules, Rust, MIT licensed.

https://github.com/hanabi-jpn/eidra
```

## Twitter/X Thread

```
1/ Your AI is leaking your secrets. Every Cursor session, every Claude Code command, every Copilot suggestion — sending your .env files, API keys, and PII to cloud servers.

I built Eidra to fix this. 🧵

2/ Eidra is a local proxy that sits between you and your AI tools.

It scans every request for 47 types of secrets — AWS keys, tokens, private keys, PII, credit cards — and masks or blocks before data leaves your machine.

[screenshot of TUI dashboard]

3/ The numbers are scary:
- Copilot repos leak secrets 40% more often
- Claude Code silently reads .env files
- MCP tools have CVSS 8.6+ vulnerabilities

You can't fix what you can't see. Eidra lets you see everything.

4/ Features:
✓ 47 scan rules (97 tests)
✓ Real-time TUI dashboard
✓ YAML policy engine (mask/block/route)
✓ Local LLM routing (Ollama)
✓ MCP firewall with tool-level ACL
✓ E2EE channels (X25519 + ChaCha20)
✓ On-device only — never phones home

5/ Built in Rust. 11 modular crates. MIT licensed.

No cloud. No account. No telemetry. Just security.

→ https://github.com/hanabi-jpn/eidra

Star it if you care about your secrets. ⭐
```
