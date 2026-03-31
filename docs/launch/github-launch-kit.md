# Eidra GitHub Launch Kit

Use this page as the primary source for GitHub launch copy.

If you want Eidra to generate the ready-to-paste files, run:

```bash
eidra launch github --write
```

To generate the default social preview image, run:

```bash
python3 scripts/generate_social_preview.py
```

It is designed to be:

- easy to paste into GitHub settings and release notes
- accurate to the current state of the repo
- simple enough for first-time visitors to understand fast

## 1. Repository About

Use one of these in the GitHub repository About box.

### Option A

`Local-first trust layer for AI development. Scan, control, and explain AI traffic before it leaves your machine.`

### Option B

`Open-source local proxy and MCP firewall for AI development workflows.`

### Option C

`A localhost trust boundary for AI tools, MCP workflows, and agentic development.`

## 2. Repository Website

If you do not have a dedicated site yet, use:

`https://github.com/<your-org-or-user>/eidra`

If you do have a site later, keep the repo About text the same and change only the website field.

## 3. Suggested Topics

Start with 8-12 topics, not 20.

Recommended first set:

- `ai-security`
- `developer-tools`
- `local-first`
- `mcp`
- `mcp-firewall`
- `proxy`
- `privacy`
- `rust`
- `agentic-ai`
- `observability`

Optional additions if they fit how the repo evolves:

- `ollama`
- `security-tooling`
- `ai-infrastructure`

## 4. Social Preview Brief

The GitHub social preview should communicate the category in one glance.

Use:

- headline: `Eidra`
- subhead: `Local-first trust layer for AI development`
- support line: `Proxy + MCP firewall + live dashboard`
- visual: the TUI dashboard screenshot or a clean diagram of `tool -> Eidra -> cloud/local`

Do not overload the image with too much text.

## 5. Pinned Assets

Before public launch, make sure these are easy to find from the repo root:

- `README.md`
- `docs/what-is-eidra.md`
- `docs/for-developers.md`
- `docs/architecture.md`
- `docs/media-kit.md`
- `docs/demo.gif`

## 6. Release Title

Recommended first release title:

`v0.1.0 — Local-First Trust Layer for AI Development`

Alternative:

`v0.1.0 — Proxy, MCP Firewall, and Trust Dashboard`

## 7. Release Notes

Use this as the first GitHub release body.

```md
## Eidra v0.1.0

Eidra is an open-source local-first trust layer for AI development.

It helps you:

- see what leaves your machine
- decide what is allowed to leave
- prove what happened later

This release packages the current core workflow into a repo that people can try, review, and build on.

### What is in this release

- local proxy for AI traffic inspection
- policy-based allow / mask / block / route decisions
- MCP gateway controls
- live terminal dashboard
- `eidra doctor` for environment checks
- `eidra setup` guidance for common environments
- machine-readable scan output
- local routing for supported OpenAI-compatible chat requests

### Good first ways to try Eidra

1. Run `eidra doctor`
2. Run `eidra scan`
3. Start the dashboard and inspect a small workflow
4. Try setup guidance for your editor, SDK, or CI path

### Who this is for

- developers using AI coding tools
- builders working with MCP workflows
- teams that want a local trust boundary before heavier governance tooling

### What to expect

This is an early but usable open-source release.

The product direction is clear, but some integrations and advanced routing paths are still evolving. Feedback on setup friction, policy ergonomics, and integration priorities is especially helpful.
```

## 8. GitHub Release Summary

Use this shorter version when you want a compact public summary.

```text
Eidra is an open-source local-first trust layer for AI development. It puts a local proxy and MCP firewall in front of AI workflows so you can inspect traffic, apply policy, and understand what happened later.
```

## 9. Release CTA

Use one of these at the end of launch posts or release notes.

### Feedback-first CTA

`If you use Cursor, Claude Code, Codex, SDK workflows, or MCP tools, I would love one honest reaction on what feels clear or confusing.`

### Builder CTA

`If you are building with MCP or agent workflows, I would especially love feedback on integration paths and policy ergonomics.`

## 10. GitHub Discussion Announcement

If Discussions are enabled, use this as the first pinned announcement.

```md
# Welcome to Eidra

Eidra is a local-first trust layer for AI development.

If you are here for the first time, the best starting points are:

- [What Is Eidra?](../what-is-eidra.md)
- [For Developers](../for-developers.md)
- [Architecture](../architecture.md)

If you try Eidra, the most useful feedback is:

- where setup felt unclear
- what workflow you wanted to protect
- what feature or integration felt missing

Thanks for taking a look.
```

## 11. Launch-Day X Post

Use this as the main public post.

```text
AI development needs a localhost trust boundary.

I’m open-sourcing Eidra: a local-first trust layer for AI development.

It sits in front of AI tools and MCP workflows, lets you inspect what is about to leave your machine, and can apply local policy to allow, mask, block, or route traffic.

If you use Cursor, Claude Code, Codex, SDK workflows, or MCP tools, I’d love honest feedback.

GitHub: https://github.com/<your-org-or-user>/eidra
```

## 12. Launch-Day Japanese Post

```text
AI開発環境に、localhost の信頼境界を置きたくて Eidra をオープンソースで公開します。

Eidra は、AIツールや MCP ワークフローの前段に置く local-first trust layer です。
外に出る前の通信を見て、ローカルの policy で allow / mask / block / route できます。

Cursor、Claude Code、Codex、SDK、MCP まわりを触っている方がいたら、率直なフィードバックをいただけると嬉しいです。

GitHub: https://github.com/<your-org-or-user>/eidra
```

## 13. Launch-Day LinkedIn Post

```text
Modern AI development is no longer just chat.

It now spans editors, coding agents, SDK workflows, MCP tools, and automation. That means developers need a clearer trust boundary around what leaves the machine.

I’m open-sourcing Eidra as a local-first trust layer for that boundary. It combines a local proxy, policy-based traffic control, an MCP firewall, and a live dashboard for understanding AI workflow behavior.

If you work on AI developer tools, agent workflows, or security-adjacent infrastructure, I would love your feedback.
```

## 14. First Comment For GitHub / Discussions / HN Cross-Posting

Use this as a softer follow-up comment after the main post.

```text
The main thing I care about right now is not reach, but clarity.

If you try Eidra, I would love to know:
- what was immediately clear
- what felt confusing
- what workflow you wanted to protect first
```

## 15. What Not To Say

Avoid these in GitHub launch copy unless the repo truly proves them:

- `the safest`
- `zero config for everyone`
- `works everywhere`
- `fully production-ready`
- `solves AI security`

Better:

- `local-first`
- `early but usable`
- `designed for gradual adoption`
- `works with existing tools`

## 16. Recommended Launch Order

1. Update GitHub About text and topics
2. Set the social preview image
3. Publish `v0.1.0`
4. Enable or pin Discussions
5. Send warm DMs
6. Post the public announcement
7. Watch the first feedback loop closely

## Companion Docs

- [Media Kit](../media-kit.md)
- [Messaging House](../messaging-house.md)
- [Outreach Playbook](../outreach.md)
- [Social Content Pack](../social-content.md)
- [Launch Checklist](../launch-checklist.md)
