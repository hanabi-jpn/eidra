# Eidra Architecture

This page explains Eidra's structure at a systems level.

## Mental Model

Eidra is not the model and not the editor.

It is the local trust layer between your AI clients and the destinations they talk to.

## High-Level Flow

```text
You / AI Tool
    |
    v
[Eidra Proxy or MCP Gateway]
    |
    +--> [Scanner]
    |       |
    |       +--> classify secrets, PII, risky patterns
    |
    +--> [Policy Engine]
    |       |
    |       +--> allow / mask / block / route
    |
    +--> [Router]
    |       |
    |       +--> local Ollama for sensitive OpenAI-compatible chat flows
    |
    +--> [Audit + TUI]
            |
            +--> local visibility and review
```

## Core Components

### Proxy

Handles AI-bound HTTP and HTTPS interception flows and becomes the main decision point for outbound traffic.

### Scanner

Applies built-in and custom rules to identify secrets, PII, and risky content.

### Policy Engine

Evaluates findings and destination context against local YAML rules.

### Router

Supports local LLM routing for sensitive OpenAI-compatible chat requests.

### Audit Store

Records local decisions and request metadata for later inspection.

### TUI

Shows live request activity, findings, and actions in a terminal dashboard.

### MCP Gateway

Adds tool-aware controls in front of MCP-connected workflows.

## Operational Modes

### Scan mode

Use `eidra scan` when you only need classification.

### Proxy mode

Use `eidra start` or `eidra dashboard` when you want inline enforcement and visibility.

### Gateway mode

Use `eidra gateway` when the main surface area is MCP tool access.

## What makes the architecture useful

- local-first enforcement
- visibility before and after policy
- separate entry points for scan, proxy, and gateway
- outputs that work for humans and automation

## What to read next

- [What Is Eidra?](what-is-eidra.md)
- [For Developers](for-developers.md)
- [Marketing Strategy](marketing-strategy.md)
