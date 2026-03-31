# Eidra For Everyone

Eidra is a safety filter for AI tools.

It sits between your AI tool and the internet, checks what is about to leave your machine, and helps stop secrets or risky actions before they go out.

## In plain language

If you only remember one thing, remember this:

> Eidra watches what your AI tools are trying to send or do, then helps you hide, stop, or reroute the risky parts.

## Why this matters

Modern AI tools are helpful because they can see a lot.
That also means they can accidentally see too much.

Examples:

- your `.env` file contains API keys
- your prompt includes internal code or customer data
- an MCP tool can read files or run commands

Most people do not see those requests directly.
Eidra makes them visible.

## What Eidra does

### 1. Shows you what is happening

Eidra can show live AI traffic in a terminal dashboard.
That means you can actually see requests, findings, and decisions.

### 2. Hides secrets before they leave

If it finds a key, token, password, or personal data, it can mask that value before the request goes out.

### 3. Blocks dangerous requests

If a request or MCP tool call looks unsafe, Eidra can block it.

### 4. Keeps some requests local

If a request is too sensitive for cloud, Eidra can route compatible chat traffic to a local model instead.

## Think of it like this

Your AI tool is fast and useful, but sometimes it shares too much.

Eidra is the person standing by the door saying:

- "This is safe to send."
- "Hide this API key first."
- "Do not run that command."
- "This one should stay local."

## Who it helps

- people trying AI coding tools for the first time
- developers using Cursor, Claude Code, Codex, or SDK apps
- teams connecting MCP tools to agents
- anyone who wants a clearer trust boundary around AI workflows

## The shortest path to try it

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra doctor
eidra setup codex
eidra dashboard
```

If you use Cursor or Claude Code instead, swap `codex` for `cursor` or `claude-code`.

## Read next

- [What Is Eidra?](what-is-eidra.md)
- [Use Cases](use-cases.md)
- [For Developers](for-developers.md)
