<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>Ve exactamente qué están filtrando tus herramientas de IA. Y detenlo.</strong></p>
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
    <a href="README.es.md"><kbd><strong>Español</strong></kbd></a>
    <br>
    <a href="README.pt.md"><kbd>Português</kbd></a>
    <a href="README.fr.md"><kbd>Français</kbd></a>
    <a href="README.de.md"><kbd>Deutsch</kbd></a>
    <a href="README.ru.md"><kbd>Русский</kbd></a>
    <a href="README.hi.md"><kbd>हिन्दी</kbd></a>
  </p>
</p>

---

> Esta página es la versión multilingüe de onboarding para GitHub. Los detalles más recientes de funciones e integraciones se mantienen en el [README en inglés](../README.md).

Claude Code puede leer tu `.env` sin pedir permiso. Los repositorios que usan Copilot filtran secretos [un 40% más a menudo](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). Las herramientas MCP también presentan [vulnerabilidades de bypass con CVSS 8.6](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html). En la práctica, tus claves API, datos de clientes y código interno pueden salir hacia servidores que no controlas sin que lo veas.

**Eidra es una trust layer local situada entre tú y tus herramientas de IA.** Analiza cada solicitud, enmascara secretos antes de que salgan del dispositivo, bloquea lo que no debería salir y te deja ver el flujo en tiempo real.

Sin nube. Sin cuenta. Todo en tu equipo.

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

## El problema

Cada vez que usas Cursor, Claude Code, Copilot, SDKs o herramientas MCP:

1. Se envía a la nube el **contexto completo de archivos**, incluidos `.env`, claves API y credenciales
2. Las **herramientas MCP** pueden tocar archivos, bases de datos y shell
3. Casi no tienes **visibilidad** de lo que realmente sale de tu máquina

Quieres trabajar más rápido con IA, pero el límite de confianza no está en tus manos.

## Qué hace Eidra

Eidra intercepta el tráfico de IA en la capa de proxy y decide localmente.

| Situación | Sin Eidra | Con Eidra |
|---|---|---|
| Clave AWS en el prompt | Se envía a la nube | `[REDACTED:api_key:a3f2]` |
| Contenido de `.env` | Se envía en silencio | Bloqueado o enmascarado |
| Clave privada SSH | Se envía a la nube | **Bloqueada** (403) |
| Datos personales | Se envían tal cual | Enmascarados para nube, permitidos para LLM local |
| Llamadas MCP | Sin límites | Controladas por política |

## Funciones principales

- **Visibilidad del flujo**: 47 reglas integradas, TUI en tiempo real y auditoría SQLite
- **Protección**: políticas YAML, enmascarado inteligente, enrutamiento a LLM local e interceptación HTTPS
- **MCP firewall**: lista blanca de servidores, ACL por herramienta, análisis de respuestas y rate limiting
- **Automatización**: `doctor --json`, `scan --json`, `config validate --json` y `setup --write`

## Inicio rápido

```bash
# Instalar
curl -sf eidra.dev/install | sh
# O compilar desde el código fuente
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# Inicializar
eidra init

# Verificar el estado local
eidra doctor

# JSON para scripts o CI
eidra doctor --json

# Instrucciones para tu entorno
eidra setup cursor
eidra setup cursor --write

# Arrancar con dashboard
eidra dashboard

# Ejecutar el gateway del firewall MCP
eidra gateway

# Escaneo puntual
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# JSON para CI u otras herramientas
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### Confiar en la CA (para la interceptación HTTPS)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# Después configura el proxy
export HTTPS_PROXY=http://127.0.0.1:8080
```

## Comandos principales

```
eidra init                    Generar la CA y la configuración inicial
eidra doctor                  Comprobar readiness y configuración efectiva
eidra doctor --json           Emitir el estado en JSON
eidra setup [target]          Mostrar pasos de integración para entornos comunes
eidra setup --write           Generar artefactos reutilizables en ~/.eidra/generated/<target>/
eidra start                   Iniciar el proxy de interceptación
eidra start -d                Iniciar proxy + TUI
eidra dashboard               Iniciar proxy + TUI
eidra gateway                 Ejecutar el gateway del firewall MCP
eidra stop                    Detener el proxy
eidra scan [file]             Escanear un archivo o stdin
eidra scan --json             Emitir findings en JSON
eidra escape                  Crear una sala cifrada sin rastro
eidra join <id> <port>        Unirse a una sala cifrada
eidra config                  Ver o editar la configuración
eidra config --json           Emitir datos de configuración en JSON
eidra config validate         Validar config y policy
eidra config validate --json  Emitir la validación en JSON
```

## Targets de setup

`eidra setup <target>` imprime pasos listos para copiar en los entornos más habituales.

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

Con `eidra setup <target> --write`, Eidra genera archivos reutilizables en `~/.eidra/generated/<target>/` sin modificar directamente tu shell o tu IDE.

## CI y automatización

`eidra scan --json`, `eidra doctor --json` y `eidra config validate --json` están pensados para CI, scripts y otras herramientas de IA que necesitan consumir la salida sin parsear texto para humanos.

## Ejemplo de policy

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

## Más detalles

Para la información más reciente sobre MCP Semantic RBAC, reglas personalizadas, canales seguros, arquitectura y roadmap, consulta el [README en inglés](../README.md). Las páginas traducidas priorizan un onboarding claro y los comandos principales.

## Contribuir

Proyecto bajo licencia MIT. Los PR son bienvenidos. Más información en [CONTRIBUTING.md](contributing.md).
