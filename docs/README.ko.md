<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>AI 도구가 무엇을 밖으로 유출하는지 정확히 보고, 바로 막으세요.</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
  <p align="center">
    <a href="../README.md"><kbd>English</kbd></a>
    <a href="README.ja.md"><kbd>日本語</kbd></a>
    <a href="README.zh.md"><kbd>简体中文</kbd></a>
    <a href="README.ko.md"><kbd><strong>한국어</strong></kbd></a>
    <a href="README.es.md"><kbd>Español</kbd></a>
    <br>
    <a href="README.pt.md"><kbd>Português</kbd></a>
    <a href="README.fr.md"><kbd>Français</kbd></a>
    <a href="README.de.md"><kbd>Deutsch</kbd></a>
    <a href="README.ru.md"><kbd>Русский</kbd></a>
    <a href="README.hi.md"><kbd>हिन्दी</kbd></a>
  </p>
</p>

---

> 이 페이지는 GitHub용 다국어 온보딩 버전입니다. 최신 기능 설명과 세부 사양은 [영문 README](../README.md)를 기준으로 유지합니다.

Claude Code는 묻지 않고 `.env`를 읽을 수 있습니다. Copilot을 사용하는 저장소는 비밀 정보가 [40% 더 자주 유출](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage)된다고 알려져 있습니다. MCP 도구에는 [CVSS 8.6 수준의 우회 취약점](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html)도 있습니다. API 키, 고객 개인정보, 내부 코드가 보이지 않는 곳에서 여러분이 통제하지 못하는 서버로 전송될 수 있습니다.

**Eidra는 여러분과 AI 도구 사이에 놓이는 로컬 trust layer입니다.** 모든 요청을 스캔하고, 민감한 정보를 기기 밖으로 보내기 전에 마스킹하며, 나가면 안 되는 것은 차단하고, 무엇이 흐르는지 실시간으로 보여줍니다.

클라우드 필요 없음. 계정 필요 없음. 모든 것이 여러분의 기기에서 동작합니다.

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

## 문제

Cursor, Claude Code, Copilot, 각종 SDK나 MCP 도구를 사용할 때마다 보통 이런 일이 벌어집니다.

1. `.env`, API 키, 데이터베이스 자격 증명을 포함한 **전체 파일 컨텍스트**가 클라우드 API로 전송됩니다
2. **MCP 도구**가 파일, 데이터베이스, 셸에 넓게 접근할 수 있습니다
3. 실제로 무엇이 밖으로 나가는지에 대한 **가시성이 거의 없습니다**

AI를 더 빠르게 쓰고 싶어도, 신뢰 경계가 내 손안에 없습니다.

## Eidra가 하는 일

Eidra는 프록시 계층에서 AI 트래픽을 받아 로컬에서 먼저 판단합니다.

| 상황 | Eidra 없음 | Eidra 사용 |
|---|---|---|
| 프롬프트 안의 AWS 키 | 클라우드로 전송 | `[REDACTED:api_key:a3f2]` |
| `.env` 내용 | 조용히 전송 | 차단 또는 마스킹 |
| SSH 개인 키 | 클라우드로 전송 | **차단** (403) |
| 개인정보 | 그대로 전송 | 클라우드용은 마스킹, 로컬 LLM용은 허용 가능 |
| MCP 도구 호출 | 제한 없음 | 정책으로 제어 |

## 핵심 기능

- **데이터 흐름 가시성**: 47개 내장 스캔 규칙, 실시간 TUI, SQLite 감사 로그
- **보호 기능**: YAML 정책, 스마트 마스킹, 로컬 LLM 라우팅, HTTPS 인터셉션
- **MCP firewall**: 서버 화이트리스트, 도구 단위 ACL, 응답 스캔, 속도 제한
- **자동화 친화성**: `doctor --json`, `scan --json`, `config validate --json`, `setup --write`

## 빠른 시작

```bash
# 설치
curl -sf eidra.dev/install | sh
# 또는 소스에서 빌드
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# 초기화
eidra init

# 로컬 준비 상태 확인
eidra doctor

# 스크립트와 CI용 JSON
eidra doctor --json

# 환경별 설정 안내
eidra setup cursor
eidra setup cursor --write

# 대시보드와 함께 시작
eidra dashboard

# MCP firewall gateway 실행
eidra gateway

# 단발성 스캔
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# CI나 다른 도구용 JSON
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### CA 신뢰 설정 (HTTPS 인터셉션용)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# 이후 프록시 설정
export HTTPS_PROXY=http://127.0.0.1:8080
```

## 핵심 명령어

```
eidra init                    CA 인증서 생성과 초기 설정
eidra doctor                  준비 상태와 유효 설정 확인
eidra doctor --json           readiness 결과를 JSON으로 출력
eidra setup [target]          일반적인 환경용 설정 단계를 출력
eidra setup --write           ~/.eidra/generated/<target>/ 아래에 재사용 가능한 파일 생성
eidra start                   인터셉트 프록시 시작
eidra start -d                프록시 + TUI 시작
eidra dashboard               프록시 + TUI 시작
eidra gateway                 MCP firewall gateway 실행
eidra stop                    프록시 중지
eidra scan [file]             파일 또는 stdin 스캔
eidra scan --json             findings를 JSON으로 출력
eidra escape                  제로 트레이스 암호화 룸 생성
eidra join <id> <port>        암호화 룸 참여
eidra config                  설정 보기/편집
eidra config --json           설정 데이터를 JSON으로 출력
eidra config validate         config와 policy 검증
eidra config validate --json  검증 결과를 JSON으로 출력
```

## 설정 대상

`eidra setup <target>`은 자주 쓰는 환경에 바로 붙여 넣을 수 있는 단계를 출력합니다.

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

`eidra setup <target> --write`를 사용하면 셸이나 IDE 파일을 직접 수정하지 않고 `~/.eidra/generated/<target>/` 아래에 재사용 가능한 결과물을 생성합니다.

## CI와 자동화

`eidra scan --json`, `eidra doctor --json`, `eidra config validate --json`은 CI, 스크립트, 다른 AI 도구가 사람이 읽는 텍스트를 다시 파싱하지 않고 바로 소비할 수 있는 출력입니다.

## 정책 예시

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

## 더 자세히 보기

MCP Semantic RBAC, 사용자 정의 스캔 규칙, 보안 채널, 아키텍처, 로드맵의 최신 내용은 [영문 README](../README.md)를 참고해 주세요. 번역 페이지는 빠른 온보딩과 핵심 명령어 전달을 우선합니다.

## 기여

MIT 라이선스 프로젝트이며 PR을 환영합니다. 자세한 내용은 [CONTRIBUTING.md](contributing.md)를 참고해 주세요。
