# What Is Eidra?

Eidra is a local safety layer for AI tools.

It sits between you and tools like Cursor, Claude Code, Codex CLI, OpenAI or Anthropic SDK-based apps, GitHub Actions, or MCP-connected workflows. Before data leaves your machine, Eidra can scan it, mask secrets, block unsafe requests, route sensitive requests to a local model, and show you what happened.

## In plain language

Eidra is the safety filter between your AI tool and the outside world.

- if something sensitive is about to leave, Eidra can hide it
- if something dangerous is about to run, Eidra can stop it
- if something should stay local, Eidra can route it locally

## Why people need it

Modern AI tools are useful because they see a lot.

That is also the problem.

- prompts may include `.env` files, API keys, or internal code
- MCP tools can touch files, databases, and shells
- most people do not see exactly what is being sent

Eidra helps you keep the speed of AI tools while getting back a local trust boundary.

## In one sentence

Eidra lets you see, control, and explain what your AI stack is sending out of your machine.

## What Eidra does

- scans requests for secrets, PII, and risky patterns
- masks or blocks sensitive content before it leaves your device
- routes some sensitive OpenAI-compatible chat requests to Ollama instead of cloud
- logs decisions locally so you can review what happened
- provides a terminal dashboard to inspect requests in real time
- runs an MCP firewall gateway for tool-level access control

## What Eidra does not do

- it does not replace your editor or model
- it does not require an account
- it does not move your traffic through another hosted SaaS layer
- it is not trying to stop you from using AI

## Who it is for

### If you are just starting with AI tools

Eidra gives you a safer default without forcing you to change your whole workflow.
If you already use Cursor, Claude Code, or Codex, the quickest path is usually one `eidra setup <target>` command plus proxy environment variables.

### If you are building AI workflows

Eidra gives you a local proxy, a policy engine, an MCP firewall, and machine-readable outputs for automation.

### If you care about security

Eidra gives you visibility, enforcement, and a clean story for "what left the machine and why."

## Real examples

### Example 1. You accidentally include secrets in a prompt

Without Eidra, your cloud model may receive the secret directly.

With Eidra, that secret can be masked or blocked first.

### Example 2. A tool call becomes risky

An MCP-connected shell or database tool can become dangerous quickly.

With Eidra, those tool calls can be constrained by policy.

### Example 3. You want local handling for sensitive prompts

Some requests are fine for cloud. Some are not.

With Eidra, sensitive OpenAI-compatible chat requests can be routed to a local Ollama model instead.

## How to try it quickly

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra doctor
eidra setup codex
eidra dashboard
```

If you use Cursor or Claude Code instead, swap `codex` for `cursor` or `claude-code`.

If you only want to test the scanner first:

```bash
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan
```

## What to read next

- [For Developers](for-developers.md)
- [Architecture](architecture.md)
- [Media Kit](media-kit.md)
