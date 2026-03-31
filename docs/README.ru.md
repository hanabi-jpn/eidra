<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>Узнайте, что именно утекает из ваших ИИ-инструментов. И остановите это.</strong></p>
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
    <a href="README.de.md"><kbd>Deutsch</kbd></a>
    <a href="README.ru.md"><kbd><strong>Русский</strong></kbd></a>
    <a href="README.hi.md"><kbd>हिन्दी</kbd></a>
  </p>
</p>

---

> Эта страница — многоязычная onboarding-версия для GitHub. Самые свежие детали по функциям и интеграциям поддерживаются в [английском README](../README.md).

Claude Code может читать ваш `.env` без запроса. Репозитории с Copilot допускают утечки секретов [на 40 % чаще](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). У MCP-инструментов также есть [уязвимости обхода уровня CVSS 8.6](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html). На практике ваши API-ключи, данные клиентов и внутренний код могут уходить на серверы вне вашего контроля, и вы этого не видите.

**Eidra — это локальный trust layer между вами и вашими ИИ-инструментами.** Он сканирует каждый запрос, маскирует секреты до выхода с устройства, блокирует то, что не должно уходить наружу, и показывает поток данных в реальном времени.

Без облака. Без аккаунта. Всё работает на вашем устройстве.

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

## Проблема

Каждый раз, когда вы используете Cursor, Claude Code, Copilot, SDK или MCP-инструменты:

1. **Полный файловый контекст**, включая `.env`, API-ключи и учётные данные, может отправляться в облачные API
2. **MCP-инструменты** могут трогать файлы, базы данных и shell
3. У вас почти нет **видимости** того, что реально покидает вашу машину

Вы хотите быстрее работать с ИИ, но граница доверия находится не у вас.

## Что делает Eidra

Eidra перехватывает ИИ-трафик на уровне прокси и принимает решение локально.

| Ситуация | Без Eidra | С Eidra |
|---|---|---|
| AWS-ключ в промпте | Уходит в облако | `[REDACTED:api_key:a3f2]` |
| Содержимое `.env` | Отправляется незаметно | Блокируется или маскируется |
| Приватный SSH-ключ | Уходит в облако | **Блокируется** (403) |
| Персональные данные | Отправляются как есть | Маскируются для облака, разрешаются для локальной LLM |
| Вызовы MCP | Без ограничений | Управляются policy |

## Основные возможности

- **Видимость потока данных**: 47 встроенных правил сканирования, TUI в реальном времени и аудит в SQLite
- **Защита**: YAML policy, умное маскирование, маршрутизация в локальную LLM и HTTPS interception
- **MCP firewall**: белый список серверов, ACL на уровне инструментов, сканирование ответов и rate limiting
- **Автоматизация**: `doctor --json`, `scan --json`, `config validate --json` и `setup --write`

## Быстрый старт

```bash
# Установка
curl -sf eidra.dev/install | sh
# Или сборка из исходников
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# Инициализация
eidra init

# Проверить локальное состояние
eidra doctor

# JSON для скриптов или CI
eidra doctor --json

# Шаги интеграции для вашей среды
eidra setup cursor
eidra setup cursor --write

# Запуск с dashboard
eidra dashboard

# Запуск gateway для MCP firewall
eidra gateway

# Разовый скан
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# JSON для CI или других инструментов
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### Доверить CA (для HTTPS interception)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# Затем настройте proxy
export HTTPS_PROXY=http://127.0.0.1:8080
```

## Основные команды

```
eidra init                    Сгенерировать CA и начальную конфигурацию
eidra doctor                  Проверить readiness и эффективную конфигурацию
eidra doctor --json           Вывести состояние в JSON
eidra setup [target]          Показать шаги интеграции для типовых окружений
eidra setup --write           Сгенерировать переиспользуемые файлы в ~/.eidra/generated/<target>/
eidra start                   Запустить proxy interception
eidra start -d                Запустить proxy + TUI
eidra dashboard               Запустить proxy + TUI
eidra gateway                 Запустить gateway MCP firewall
eidra stop                    Остановить proxy
eidra scan [file]             Просканировать файл или stdin
eidra scan --json             Вывести findings в JSON
eidra escape                  Создать зашифрованную комнату без следов
eidra join <id> <port>        Присоединиться к зашифрованной комнате
eidra config                  Просмотреть или изменить конфигурацию
eidra config --json           Вывести конфигурацию в JSON
eidra config validate         Проверить config и policy
eidra config validate --json  Вывести валидацию в JSON
```

## Цели setup

`eidra setup <target>` выводит готовые к копированию шаги для типовых окружений.

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

С `eidra setup <target> --write` Eidra создаёт переиспользуемые файлы в `~/.eidra/generated/<target>/`, не меняя напрямую ваши shell- или IDE-конфиги.

## CI и автоматизация

`eidra scan --json`, `eidra doctor --json` и `eidra config validate --json` удобны для CI, скриптов и других ИИ-инструментов, которым нужен машинно-читаемый вывод без парсинга текста для человека.

## Пример policy

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

## Подробнее

За самыми свежими деталями по MCP Semantic RBAC, пользовательским правилам, защищённым каналам, архитектуре и roadmap обращайтесь к [английскому README](../README.md). Переводные страницы в первую очередь делают быстрым onboarding и понятными ключевые команды.

## Участие

Проект распространяется по лицензии MIT. PR приветствуются. Подробнее в [CONTRIBUTING.md](contributing.md).
