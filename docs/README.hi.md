[English](../README.md) | [日本語](README.ja.md) | [简体中文](README.zh.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Русский](README.ru.md) | [हिन्दी](README.hi.md)

<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>देखिए आपके AI टूल्स से क्या-क्या लीक हो रहा है। फिर उसे रोकिए।</strong></p>
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

> यह पेज GitHub के लिए बहुभाषी ऑनबोर्डिंग संस्करण है। नए फीचर्स और इंटीग्रेशन की सबसे ताज़ा जानकारी [अंग्रेज़ी README](../README.md) में रखी जाती है।

Claude Code बिना पूछे आपका `.env` पढ़ सकता है। Copilot इस्तेमाल करने वाली repositories में secrets [40% ज़्यादा बार](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage) लीक होते हैं। MCP टूल्स में [CVSS 8.6 स्तर की bypass कमजोरियाँ](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html) भी मिली हैं। व्यवहार में आपकी API keys, ग्राहक डेटा और आंतरिक कोड उन servers तक जा सकते हैं जिन पर आपका नियंत्रण नहीं है, और आपको पता भी नहीं चलता।

**Eidra आपके और आपके AI टूल्स के बीच बैठने वाली एक local trust layer है।** यह हर request को scan करता है, secrets को device से बाहर जाने से पहले mask करता है, जो बाहर नहीं जाना चाहिए उसे block करता है, और पूरा flow real time में दिखाता है।

कोई cloud नहीं। कोई account नहीं। सब कुछ आपके device पर।

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

## समस्या

जब भी आप Cursor, Claude Code, Copilot, SDKs या MCP टूल्स का इस्तेमाल करते हैं:

1. `.env`, API keys और credentials सहित **पूरा file context** cloud APIs तक जा सकता है
2. **MCP टूल्स** files, databases और shell को छू सकते हैं
3. आपकी मशीन से वास्तव में क्या बाहर जा रहा है, इसकी **visibility** लगभग नहीं होती

आप AI के साथ तेज़ काम करना चाहते हैं, लेकिन trust boundary आपके हाथ में नहीं रहती।

## Eidra क्या करता है

Eidra proxy layer पर AI traffic को intercept करता है और locally फैसला लेता है।

| स्थिति | Eidra के बिना | Eidra के साथ |
|---|---|---|
| Prompt में AWS key | Cloud को भेजी गई | `[REDACTED:api_key:a3f2]` |
| `.env` सामग्री | चुपचाप भेजी गई | Block या mask |
| SSH private key | Cloud को भेजी गई | **Block** (403) |
| Personal data | जैसे का तैसा भेजा गया | Cloud के लिए masked, local LLM के लिए allowed |
| MCP calls | बिना सीमा | Policy से नियंत्रित |

## मुख्य फीचर्स

- **Data flow visibility**: 47 built-in scan rules, real-time TUI और SQLite audit log
- **Protection**: YAML policy, smart masking, local LLM routing और HTTPS interception
- **MCP firewall**: server whitelist, tool-level ACL, response scanning और rate limiting
- **Automation**: `doctor --json`, `scan --json`, `config validate --json` और `setup --write`

## त्वरित शुरुआत

```bash
# Install
curl -sf eidra.dev/install | sh
# या source से build करें
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# Initialize
eidra init

# Local readiness जाँचें
eidra doctor

# Scripts या CI के लिए JSON
eidra doctor --json

# अपने environment के लिए integration steps
eidra setup cursor
eidra setup cursor --write

# Dashboard के साथ शुरू करें
eidra dashboard

# MCP firewall gateway चलाएँ
eidra gateway

# एक बार scan करें
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# CI या दूसरे tools के लिए JSON
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### CA पर भरोसा करें (HTTPS interception के लिए)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# फिर proxy सेट करें
export HTTPS_PROXY=http://127.0.0.1:8080
```

## मुख्य कमांड

```
eidra init                    CA और शुरुआती configuration बनाएं
eidra doctor                  readiness और effective configuration जाँचें
eidra doctor --json           status को JSON में निकालें
eidra setup [target]          सामान्य environments के लिए integration steps दिखाएँ
eidra setup --write           ~/.eidra/generated/<target>/ में reusable artifacts बनाएं
eidra start                   interception proxy शुरू करें
eidra start -d                proxy + TUI शुरू करें
eidra dashboard               proxy + TUI शुरू करें
eidra gateway                 MCP firewall gateway चलाएँ
eidra stop                    proxy बंद करें
eidra scan [file]             file या stdin scan करें
eidra scan --json             findings को JSON में निकालें
eidra escape                  zero-trace encrypted room बनाएं
eidra join <id> <port>        encrypted room में जुड़ें
eidra config                  configuration देखें या बदलें
eidra config --json           configuration data को JSON में निकालें
eidra config validate         config और policy validate करें
eidra config validate --json  validation को JSON में निकालें
```

## Setup targets

`eidra setup <target>` आम environments के लिए copy-paste करने लायक steps देता है।

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

`eidra setup <target> --write` के साथ Eidra सीधे आपकी shell या IDE files को छुए बिना `~/.eidra/generated/<target>/` में reusable files बनाता है।

## CI और automation

`eidra scan --json`, `eidra doctor --json` और `eidra config validate --json` CI, scripts और दूसरे AI tools के लिए बने हैं, ताकि उन्हें human-readable text parse न करना पड़े।

## Policy उदाहरण

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

## और अधिक जानकारी

MCP Semantic RBAC, custom rules, secure channels, architecture और roadmap की सबसे नई जानकारी के लिए [अंग्रेज़ी README](../README.md) देखें। अनुवादित पेज तेज़ onboarding और मुख्य commands को प्राथमिकता देते हैं।

## योगदान

यह MIT लाइसेंस वाला प्रोजेक्ट है। PRs का स्वागत है। अधिक जानकारी [CONTRIBUTING.md](contributing.md) में है।
