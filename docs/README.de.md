<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>Sehen Sie genau, was Ihre KI-Tools preisgeben. Und stoppen Sie es.</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
  <p align="center">
    <a href="../README.md"><kbd>English</kbd></a>
    <a href="README.ja.md"><kbd>日本語</kbd></a>
    <a href="README.zh.md"><kbd>简体中文</kbd></a>
    <a href="README.ko.md"><kbd>한국어</kbd></a>
    <a href="README.es.md"><kbd>Español</kbd></a>
    <br>
    <a href="README.pt.md"><kbd>Português</kbd></a>
    <a href="README.fr.md"><kbd>Français</kbd></a>
    <a href="README.de.md"><kbd><strong>Deutsch</strong></kbd></a>
    <a href="README.ru.md"><kbd>Русский</kbd></a>
    <a href="README.hi.md"><kbd>हिन्दी</kbd></a>
  </p>
</p>

---

> Diese Seite ist die mehrsprachige Onboarding-Version für GitHub. Die neuesten Details zu Funktionen und Integrationen werden im [englischen README](../README.md) gepflegt.

Claude Code kann Ihre `.env` lesen, ohne zu fragen. Repositories mit Copilot leaken Geheimnisse [40 % häufiger](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). MCP-Tools weisen außerdem [CVSS-8.6-Bypass-Schwachstellen](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html) auf. In der Praxis können API-Schlüssel, Kundendaten und interner Code auf Server gelangen, die Sie nicht kontrollieren, ohne dass Sie es sehen.

**Eidra ist eine lokale Trust Layer zwischen Ihnen und Ihren KI-Tools.** Es scannt jede Anfrage, maskiert Geheimnisse bevor sie das Gerät verlassen, blockiert unerwünschte Abflüsse und macht den Datenfluss in Echtzeit sichtbar.

Keine Cloud. Kein Konto. Alles bleibt auf Ihrem Gerät.

Einfach gesagt: Eidra ist ein Sicherheitsfilter für KI-Tools. Es beobachtet, was hinausgehen oder ausgeführt werden soll, und hilft dabei, riskante Teile zu verbergen, zu stoppen oder lokal zu halten.

<p align="center">
  <img src="demo.gif" alt="Eidra TUI Dashboard" width="800">
</p>

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra doctor --json
eidra setup codex --write
eidra dashboard
```

---

## Lesehilfe

- Für die einfachste Erklärung: [For Everyone](for-everyone.md)
- Für konkrete Beispiele: [Use Cases](use-cases.md)
- Für die neuesten technischen Details: [englisches README](../README.md) und [For Developers](for-developers.md)

---

## Das Problem

Immer wenn Sie Cursor, Claude Code, Copilot, SDKs oder MCP-Tools verwenden:

1. Der **vollständige Dateikontext**, einschließlich `.env`, API-Schlüsseln und Zugangsdaten, kann an Cloud-APIs gesendet werden
2. **MCP-Tools** können auf Dateien, Datenbanken und die Shell zugreifen
3. Sie haben fast keine **Sichtbarkeit** darauf, was Ihre Maschine wirklich verlässt

Sie möchten KI schneller einsetzen, aber die Vertrauensgrenze liegt nicht in Ihrer Hand.

## Was Eidra macht

Eidra fängt KI-Traffic auf Proxy-Ebene ab und entscheidet lokal.

| Situation | Ohne Eidra | Mit Eidra |
|---|---|---|
| AWS-Schlüssel im Prompt | In die Cloud gesendet | `[REDACTED:api_key:a3f2]` |
| Inhalt von `.env` | Still gesendet | Blockiert oder maskiert |
| Privater SSH-Schlüssel | In die Cloud gesendet | **Blockiert** (403) |
| Personenbezogene Daten | Unverändert gesendet | Für Cloud maskiert, für lokales LLM erlaubt |
| MCP-Aufrufe | Uneingeschränkt | Per Policy gesteuert |

## Zentrale Funktionen

- **Datenfluss-Sichtbarkeit**: 47 integrierte Scan-Regeln, Echtzeit-TUI und SQLite-Audit-Log
- **Schutz**: YAML-Policies, intelligentes Maskieren, lokales LLM-Routing und HTTPS-Interception
- **MCP firewall**: Server-Whitelist, ACL pro Tool, Response-Scanning und Rate Limiting
- **Automatisierung**: `doctor --json`, `scan --json`, `config validate --json` und `setup --write`

## Schnellstart

```bash
# Installieren
curl -sf eidra.dev/install | sh
# Oder aus dem Quellcode bauen
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# Initialisieren
eidra init

# Lokalen Zustand prüfen
eidra doctor

# JSON für Skripte oder CI
eidra doctor --json

# Integrationsschritte für Ihre Umgebung
eidra setup cursor
eidra setup cursor --write

# Mit Dashboard starten
eidra dashboard

# Das MCP-firewall-Gateway starten
eidra gateway

# Einzelner Scan
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# JSON für CI oder andere Tools
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### Der CA vertrauen (für HTTPS-Interception)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# Danach den Proxy setzen
export HTTPS_PROXY=http://127.0.0.1:8080
```

## Kernbefehle

```
eidra init                    CA und Initialkonfiguration erzeugen
eidra doctor                  Readiness und effektive Konfiguration prüfen
eidra doctor --json           Status als JSON ausgeben
eidra setup [target]          Integrationsschritte für gängige Umgebungen anzeigen
eidra setup --write           Wiederverwendbare Artefakte unter ~/.eidra/generated/<target>/ erzeugen
eidra start                   Interception-Proxy starten
eidra start -d                Proxy + TUI starten
eidra dashboard               Proxy + TUI starten
eidra gateway                 Das MCP-firewall-Gateway ausführen
eidra stop                    Proxy stoppen
eidra scan [file]             Datei oder stdin scannen
eidra scan --json             Findings als JSON ausgeben
eidra escape                  Einen spurlosen verschlüsselten Raum erstellen
eidra join <id> <port>        Einem verschlüsselten Raum beitreten
eidra config                  Konfiguration anzeigen oder bearbeiten
eidra config --json           Konfigurationsdaten als JSON ausgeben
eidra config validate         Config und Policy validieren
eidra config validate --json  Validierung als JSON ausgeben
```

## Setup-Ziele

`eidra setup <target>` gibt direkt kopierbare Schritte für gängige Umgebungen aus.

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

Mit `eidra setup <target> --write` erzeugt Eidra wiederverwendbare Dateien unter `~/.eidra/generated/<target>/`, ohne Ihre Shell- oder IDE-Dateien direkt zu verändern.

## CI und Automatisierung

`eidra scan --json`, `eidra doctor --json` und `eidra config validate --json` sind für CI, Skripte und andere KI-Tools gedacht, die Ausgaben maschinell konsumieren möchten, ohne menschenlesbaren Text zu parsen.

## Beispiel-Policy

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

## Mehr Details

Die neuesten Informationen zu MCP Semantic RBAC, benutzerdefinierten Regeln, sicheren Kanälen, Architektur und Roadmap finden Sie im [englischen README](../README.md). Die Übersetzungen priorisieren klares Onboarding und die wichtigsten Befehle.

## Mitwirken

MIT-lizenziertes Projekt. PRs sind willkommen. Mehr dazu in [CONTRIBUTING.md](contributing.md).
