[English](../README.md) | [日本語](README.ja.md) | [简体中文](README.zh.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Русский](README.ru.md) | [हिन्दी](README.hi.md)

<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>看清你的 AI 工具到底泄露了什么，然后阻止它。</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
  <p align="center">
    <a href="../README.md">English</a> ·
    <a href="README.ja.md">日本語</a> ·
    <a href="README.zh.md">简体中文</a> ·
    <a href="README.ko.md">한국어</a> ·
    <a href="README.es.md">Español</a> ·
    <a href="README.pt.md">Português</a> ·
    <a href="README.fr.md">Français</a> ·
    <a href="README.de.md">Deutsch</a> ·
    <a href="README.ru.md">Русский</a> ·
    <a href="README.hi.md">हिन्दी</a>
  </p>
</p>

---

> 这个页面是面向 GitHub 的多语言快速上手版本。最新的功能细节和完整说明以 [英文 README](../README.md) 为准。

Claude Code 会在不询问的情况下读取你的 `.env`。使用 Copilot 的仓库发生密钥泄露的概率据称[高出 40%](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage)。MCP 工具还存在 [CVSS 8.6 级别的绕过漏洞](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html)。很多时候，API 密钥、客户隐私数据和内部代码会在你看不见的情况下被发送到你无法控制的服务器。

**Eidra 是位于你与 AI 工具之间的本地 trust layer。** 它会扫描每一次请求，在敏感信息离开设备前完成脱敏，阻止不该发送的内容，并实时展示数据流向。

无需云端。无需账号。全部在你的设备上完成。

<p align="center">
  <img src="demo.gif" alt="Eidra TUI Dashboard" width="800">
</p>

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra doctor --json
eidra setup shell --write
eidra dashboard
```

---

## 问题

每次你使用 Cursor、Claude Code、Copilot、各种 SDK 或 MCP 工具时，通常都会发生这些事情：

1. 包含 `.env`、API 密钥、数据库凭据的**完整文件上下文**被发送到云端 API
2. **MCP 工具**可能直接访问文件、数据库和 shell
3. 你对真正传出去的内容几乎**没有可见性**

你想更快地用 AI，但信任边界并不在自己手里。

## Eidra 做什么

Eidra 在代理层接住 AI 流量，并在本地先做判断。

| 场景 | 没有 Eidra | 使用 Eidra |
|---|---|---|
| 提示词中的 AWS 密钥 | 发送到云端 | `[REDACTED:api_key:a3f2]` |
| `.env` 内容 | 静默发送 | 阻止或脱敏 |
| SSH 私钥 | 发送到云端 | **阻止** (403) |
| 个人信息 | 直接发送 | 面向云端时脱敏，面向本地 LLM 时可放行 |
| MCP 工具调用 | 无限制 | 由策略控制 |

## 核心功能

- **数据流可视化**: 47 条内置扫描规则、实时 TUI、SQLite 审计日志
- **保护能力**: YAML 策略、智能脱敏、本地 LLM 路由、HTTPS 拦截
- **MCP firewall**: 服务器白名单、工具级 ACL、响应扫描、速率限制
- **自动化友好**: `doctor --json`、`scan --json`、`config validate --json`、`setup --write`

## 快速开始

```bash
# 安装
curl -sf eidra.dev/install | sh
# 或从源码构建
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# 初始化
eidra init

# 检查本地环境是否就绪
eidra doctor

# 面向脚本和 CI 的 JSON 输出
eidra doctor --json

# 打印当前环境的接入步骤
eidra setup cursor
eidra setup cursor --write

# 启动带仪表盘的代理
eidra dashboard

# 启动 MCP firewall gateway
eidra gateway

# 直接扫描一次
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# 提供给 CI 或其他工具的 JSON
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### 信任 CA 证书（用于 HTTPS 拦截）

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# 然后设置代理
export HTTPS_PROXY=http://127.0.0.1:8080
```

## 核心命令

```
eidra init                    生成 CA 证书并初始化配置
eidra doctor                  检查 readiness 和当前有效配置
eidra doctor --json           以 JSON 输出 readiness 结果
eidra setup [target]          输出常见环境的接入步骤
eidra setup --write           在 ~/.eidra/generated/<target>/ 生成可复用文件
eidra start                   启动拦截代理
eidra start -d                启动代理 + TUI
eidra dashboard               启动代理 + TUI
eidra gateway                 启动 MCP firewall gateway
eidra stop                    停止代理
eidra scan [file]             扫描文件或 stdin
eidra scan --json             以 JSON 输出 findings
eidra escape                  创建零痕迹加密房间
eidra join <id> <port>        加入加密房间
eidra config                  查看或编辑配置
eidra config --json           以 JSON 输出配置数据
eidra config validate         校验 config 和 policy
eidra config validate --json  以 JSON 输出校验结果
```

## 支持的接入目标

`eidra setup <target>` 可以直接给出常见环境的复制即用步骤。

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

使用 `eidra setup <target> --write` 时，Eidra 会把可复用的配置文件写到 `~/.eidra/generated/<target>/`，而不是直接修改你的 shell 或 IDE。

## CI 与自动化

`eidra scan --json`、`eidra doctor --json`、`eidra config validate --json` 适合直接接入 CI、脚本和其他 AI 工具，无需再去解析面向人类的文本输出。

## 策略示例

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

## 更多内容

关于 MCP Semantic RBAC、自定义扫描规则、安全通道、架构和路线图的最新细节，请查看 [英文 README](../README.md)。翻译页优先保证“上手快”和“核心命令清晰”。

## 贡献

项目使用 MIT 许可证，欢迎提交 PR。更多信息请参见 [CONTRIBUTING.md](contributing.md)。
