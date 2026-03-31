# Eidra Media Kit

Use this page when writing about Eidra, describing it in a newsletter, or introducing it on social platforms.

## One-Line Description

Eidra is a local-first trust layer for AI development that lets you see, control, and explain what your AI tools are sending out of your machine.

## Short Description

Eidra is an open-source local proxy and MCP firewall for AI workflows. It scans requests for secrets and risky content, applies policy to allow, mask, block, or route traffic, and gives developers a real-time dashboard plus machine-readable outputs for automation.

## Longer Description

Eidra sits between developers and modern AI tooling such as editors, SDK workflows, and MCP-connected agents. Before data leaves the machine, Eidra can classify sensitive content, apply local policy, route some sensitive OpenAI-compatible chat traffic to Ollama, and log what happened locally. The goal is not to slow AI adoption down. The goal is to give AI development a trustworthy localhost boundary.

## Tagline Options

- Your AI stack needs a localhost trust layer.
- See what leaves. Decide what goes. Prove what happened.
- Local-first trust for AI development.

## Who It Is For

- developers using Cursor, Claude Code, Copilot, or SDK-based AI workflows
- builders wiring AI agents to MCP tools
- security-minded teams that want local visibility and enforcement
- creators and educators explaining safe AI development

## Key Talking Points

- local-first, not another hosted gatekeeper
- works with existing tools instead of replacing them
- visibility plus enforcement, not fear-only messaging
- useful for both solo developers and automation-heavy workflows
- includes both human-facing and machine-readable surfaces

## Why Now

AI development has shifted from isolated chat boxes to connected workflows across editors, SDKs, agents, MCP tools, and CI.

That makes "what leaves the machine" a much more important product question than before.

## Facts To Mention Carefully

- open source and MIT licensed
- local proxy, local audit log, local dashboard
- MCP firewall gateway with tool-aware controls
- setup guidance for common environments
- JSON output for CI and automation

Avoid implying features that do not exist yet.

## Example Headlines

- The local trust layer for AI coding tools
- A proxy and MCP firewall for safer AI development
- Open-source guardrails for what your AI stack sends out
- A localhost boundary for agentic development workflows

## FAQ

### Is Eidra trying to replace AI tools?

No. Eidra is designed to sit in front of existing tools and make them easier to trust.

### Does it require a cloud account?

No. Eidra is designed to be local-first.

### Is this only for security teams?

No. It is useful for individual developers, builders working with MCP, and teams that need a clearer trust boundary.

### What is the easiest way to try it?

Use the quickstart in the main README or start with `eidra scan`, `eidra doctor`, and `eidra setup`.

## Recommended Assets

- the terminal dashboard GIF
- one static screenshot of the dashboard
- one architecture diagram
- one beginner explanation
- one developer explanation

## Recommended Links

- [What Is Eidra?](what-is-eidra.md)
- [For Developers](for-developers.md)
- [Architecture](architecture.md)
- [README](../README.md)
- [Messaging House](messaging-house.md)
- [Outreach Playbook](outreach.md)
