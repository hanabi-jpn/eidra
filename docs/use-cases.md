# Eidra Use Cases

These are the three fastest ways to understand why someone installs Eidra.

## 1. Your coding agent tries to send `.env`

You ask Claude Code, Codex, Cursor, or another coding tool to debug a project.
The tool grabs broad file context, including `.env`, API keys, or database credentials.

Without Eidra:

- that context can go straight to a cloud API
- you often do not see the raw request

With Eidra:

- the outbound request is scanned before it leaves your machine
- secrets can be masked or blocked by policy
- the dashboard and audit log show what happened

Typical outcome:

```text
.env contents detected -> mask or block before egress
```

## 2. An MCP tool call becomes dangerous

You connect an MCP server for shell, filesystem, or database access.
Your AI agent now has tool power, not just prompt context.

Without Eidra:

- `run_command("rm -rf /")`
- `execute_sql("DROP TABLE users")`
- `read_file("/etc/shadow")`

can reach the tool server unless that server implements strong controls itself.

With Eidra:

- MCP servers can be allowlisted
- individual tools can be allowed or blocked
- tool arguments can be inspected for destructive patterns
- sensitive tool responses can be scanned on the way back

Typical outcome:

```text
execute_sql("DROP TABLE users") -> blocked by MCP policy
```

## 3. A request is too sensitive for cloud

Some requests are fine to send to a hosted model.
Some are not.

Without Eidra:

- you choose between speed and control

With Eidra:

- the same request can be scanned locally
- policy can decide whether to allow, mask, block, or route it
- sensitive OpenAI-compatible chat traffic can be sent to Ollama instead

Typical outcome:

```text
PII or internal code detected -> route request to local model
```

## What people usually do first

```bash
eidra init
eidra doctor
eidra setup codex
eidra dashboard
```

Then they try one of these:

- paste a test secret into `eidra scan`
- run their editor or agent through the proxy
- start `eidra gateway` in front of MCP tools

## Read next

- [What Is Eidra?](what-is-eidra.md)
- [For Developers](for-developers.md)
- [Architecture](architecture.md)
