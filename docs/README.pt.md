<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>Veja exatamente o que suas ferramentas de IA estão vazando. E interrompa isso.</strong></p>
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
    <a href="README.pt.md"><kbd><strong>Português</strong></kbd></a>
    <a href="README.fr.md"><kbd>Français</kbd></a>
    <a href="README.de.md"><kbd>Deutsch</kbd></a>
    <a href="README.ru.md"><kbd>Русский</kbd></a>
    <a href="README.hi.md"><kbd>हिन्दी</kbd></a>
  </p>
</p>

---

> Esta página é a versão multilíngue de onboarding para o GitHub. Os detalhes mais recentes sobre recursos e integrações ficam no [README em inglês](../README.md).

O Claude Code pode ler seu `.env` sem pedir permissão. Repositórios que usam Copilot vazam segredos [40% mais vezes](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). Ferramentas MCP também apresentam [vulnerabilidades de bypass com CVSS 8.6](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html). Na prática, suas chaves de API, dados de clientes e código interno podem sair para servidores fora do seu controle sem que você perceba.

**Eidra é uma trust layer local entre você e suas ferramentas de IA.** Ele inspeciona cada requisição, mascara segredos antes de eles saírem do dispositivo, bloqueia o que não deveria sair e mostra o fluxo em tempo real.

Sem nuvem. Sem conta. Tudo roda no seu dispositivo.

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

## O problema

Sempre que você usa Cursor, Claude Code, Copilot, SDKs ou ferramentas MCP:

1. Todo o **contexto de arquivos**, incluindo `.env`, chaves de API e credenciais, pode ser enviado para APIs em nuvem
2. As **ferramentas MCP** podem tocar arquivos, bancos de dados e shell
3. Você quase não tem **visibilidade** do que realmente está saindo da sua máquina

Você quer usar IA com velocidade, mas o limite de confiança não está nas suas mãos.

## O que o Eidra faz

O Eidra intercepta o tráfego de IA na camada de proxy e decide localmente.

| Situação | Sem Eidra | Com Eidra |
|---|---|---|
| Chave AWS no prompt | Enviada para a nuvem | `[REDACTED:api_key:a3f2]` |
| Conteúdo de `.env` | Enviado em silêncio | Bloqueado ou mascarado |
| Chave privada SSH | Enviada para a nuvem | **Bloqueada** (403) |
| Dados pessoais | Enviados sem proteção | Mascarados para nuvem, permitidos para LLM local |
| Chamadas MCP | Sem restrição | Controladas por política |

## Recursos principais

- **Visibilidade de fluxo**: 47 regras integradas, TUI em tempo real e trilha de auditoria em SQLite
- **Proteção**: políticas YAML, mascaramento inteligente, roteamento para LLM local e interceptação HTTPS
- **MCP firewall**: whitelist de servidores, ACL por ferramenta, varredura de respostas e rate limiting
- **Automação**: `doctor --json`, `scan --json`, `config validate --json` e `setup --write`

## Início rápido

```bash
# Instalar
curl -sf eidra.dev/install | sh
# Ou compilar a partir do código-fonte
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# Inicializar
eidra init

# Verificar o estado local
eidra doctor

# JSON para scripts ou CI
eidra doctor --json

# Instruções para o seu ambiente
eidra setup cursor
eidra setup cursor --write

# Iniciar com dashboard
eidra dashboard

# Executar o gateway do firewall MCP
eidra gateway

# Varredura pontual
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# JSON para CI ou outras ferramentas
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### Confiar na CA (para interceptação HTTPS)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# Depois configure o proxy
export HTTPS_PROXY=http://127.0.0.1:8080
```

## Comandos principais

```
eidra init                    Gerar a CA e a configuração inicial
eidra doctor                  Verificar readiness e configuração efetiva
eidra doctor --json           Emitir o estado em JSON
eidra setup [target]          Mostrar passos de integração para ambientes comuns
eidra setup --write           Gerar artefatos reutilizáveis em ~/.eidra/generated/<target>/
eidra start                   Iniciar o proxy de interceptação
eidra start -d                Iniciar proxy + TUI
eidra dashboard               Iniciar proxy + TUI
eidra gateway                 Executar o gateway do firewall MCP
eidra stop                    Parar o proxy
eidra scan [file]             Escanear um arquivo ou stdin
eidra scan --json             Emitir findings em JSON
eidra escape                  Criar uma sala criptografada sem rastros
eidra join <id> <port>        Entrar em uma sala criptografada
eidra config                  Ver ou editar a configuração
eidra config --json           Emitir dados de configuração em JSON
eidra config validate         Validar config e policy
eidra config validate --json  Emitir a validação em JSON
```

## Targets de setup

`eidra setup <target>` imprime passos prontos para copiar nos ambientes mais comuns.

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

Com `eidra setup <target> --write`, o Eidra gera arquivos reutilizáveis em `~/.eidra/generated/<target>/` sem editar diretamente seu shell ou sua IDE.

## CI e automação

`eidra scan --json`, `eidra doctor --json` e `eidra config validate --json` servem para CI, scripts e outras ferramentas de IA consumirem a saída sem precisar interpretar texto voltado para humanos.

## Exemplo de policy

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

## Mais detalhes

Para ver as informações mais recentes sobre MCP Semantic RBAC, regras personalizadas, canais seguros, arquitetura e roadmap, consulte o [README em inglês](../README.md). As páginas traduzidas priorizam um onboarding claro e os comandos centrais.

## Contribuir

Projeto sob licença MIT. PRs são bem-vindos. Mais detalhes em [CONTRIBUTING.md](contributing.md).
