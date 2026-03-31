<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>AIツールが何を外へ漏らしているか、正確に見る。そして止める。</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
  <p align="center">
    <a href="../README.md"><kbd>English</kbd></a>
    <a href="README.ja.md"><kbd><strong>日本語</strong></kbd></a>
    <a href="README.zh.md"><kbd>简体中文</kbd></a>
    <a href="README.ko.md"><kbd>한국어</kbd></a>
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

> このページは GitHub 向けの多言語オンボーディング版です。最新の機能差分や詳細仕様は [英語 README](../README.md) を正本として追従します。

Claude Code は確認なしに `.env` を読み取ります。Copilot を使うリポジトリでは秘密情報の漏洩が [40% 増える](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage) と報告されています。MCP ツールには [CVSS 8.6 のバイパス脆弱性](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html) もあります。気づかないうちに、API キー、顧客データ、社内コードが自分で管理できない先へ送られているのが今の現実です。

**Eidra は、あなたと AI ツールの間に入るローカル trust layer です。** すべてのリクエストをスキャンし、秘密情報を端末の外へ出る前にマスクし、送るべきでないものを止め、何が流れているかをリアルタイムで見せます。

クラウド不要。アカウント不要。すべてデバイス上で完結します。

ひとことで言うと、Eidra は「AI の見張り番」です。AI が送ろうとしている内容や実行しようとしていることを見て、危ない部分だけ先に対応します。

いまは **Cursor**、**Claude Code**、**Codex CLI**、**OpenAI / Anthropic 系 SDK アプリ**、**GitHub Actions**、**MCP ツールチェーン** にすぐ組み込めます。
`HTTP_PROXY` / `HTTPS_PROXY` が使えるツールなら、基本的に Eidra を前段に置けます。

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

## 読み方ガイド

- いちばんやさしい説明から入りたいなら [はじめての Eidra](for-everyone.ja.md)
- 具体例を見たいなら [ユースケース集](use-cases.ja.md)
- 技術寄りに知りたいなら [英語 README](../README.md) と [For Developers](for-developers.md)

---

## 30秒で分かる例

### 1. コーディングエージェントが `.env` を送ろうとする

```text
Eidra なし: prompt + .env の内容 -> cloud API
Eidra あり: prompt + [REDACTED:api_key:a3f2] -> cloud API
```

### 2. MCP の tool call が破壊的になる

```text
execute_sql("DROP TABLE users") -> MCP policy で block
run_command("rm -rf /")         -> MCP policy で block
```

### 3. クラウドに送りたくないリクエストがある

```text
PII や社内コードを検出 -> local Ollama model へ route
```

長めの具体例は [ユースケース集](use-cases.ja.md) にまとめています。

---

## 課題

Cursor、Claude Code、Codex、Copilot、各種 SDK や MCP ツールを使うたびに、次のことが起きます。

1. `.env`、API キー、データベース認証情報を含む**ファイル文脈全体**がクラウド API へ送られる
2. **MCP ツール**がファイル、データベース、シェルに広く触れられる
3. 実際に何が外へ出ているかの**可視性がほぼない**

AI を速く使いたいのに、信頼境界が自分の手元にありません。

## Eidra がやること

Eidra はプロキシ層で AI トラフィックを受け止め、ローカル側で判断します。

| 状況 | Eidra なし | Eidra あり |
|---|---|---|
| プロンプト内の AWS キー | クラウドへ送信 | `[REDACTED:api_key:a3f2]` |
| `.env` の内容 | そのまま送信 | ブロックまたはマスク |
| SSH 秘密鍵 | クラウドへ送信 | **ブロック** (403) |
| 個人情報 | そのまま送信 | クラウド向けはマスク、ローカル LLM 向けは許可可能 |
| MCP ツール呼び出し | 制限なし | ポリシーで制御 |

## 主な機能

- **データフロー可視化**: 47 の組み込みスキャンルール、リアルタイム TUI、SQLite 監査ログ
- **保護**: YAML ポリシー、スマートマスキング、ローカル LLM ルーティング、HTTPS 傍受
- **MCP firewall**: サーバーホワイトリスト、ツール単位 ACL、レスポンススキャン、レート制限
- **自動化しやすさ**: `doctor --json`、`scan --json`、`config validate --json`、`setup --write`

## クイックスタート

```bash
# インストール
curl -sf eidra.dev/install | sh
# またはソースからビルド
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# 初期化
eidra init

# ローカルセットアップ確認
eidra doctor

# スクリプトや CI 向け JSON
eidra doctor --json

# 環境ごとのセットアップ案内
eidra setup codex
eidra setup codex --write

# ダッシュボード付きで起動
eidra dashboard

# MCP firewall gateway を起動
eidra gateway

# 単発スキャン
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# CI や他ツール向け JSON
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### CA 証明書を信頼する（HTTPS 傍受用）

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# その後プロキシを設定
export HTTPS_PROXY=http://127.0.0.1:8080
```

## 主要コマンド

```
eidra init                    CA 証明書生成と初期設定
eidra doctor                  起動前チェックと有効設定の確認
eidra doctor --json           readiness 情報を JSON で出力
eidra setup [target]          一般的な環境向けセットアップ手順を表示
eidra setup --write           ~/.eidra/generated/<target>/ に成果物を生成
eidra start                   傍受プロキシを起動
eidra start -d                プロキシ + TUI を起動
eidra dashboard               プロキシ + TUI を起動
eidra gateway                 MCP firewall gateway を起動
eidra stop                    プロキシを停止
eidra scan [file]             ファイルまたは stdin をスキャン
eidra scan --json             findings を JSON で出力
eidra escape                  ゼロトレース暗号化ルームを作成
eidra join <id> <port>        暗号化ルームに参加
eidra config                  設定を表示・編集
eidra config --json           設定情報を JSON で出力
eidra config validate         config と policy を検証
eidra config validate --json  検証結果を JSON で出力
```

## セットアップ対象

`eidra setup <target>` は、よく使う環境へ貼り付けやすい導線を出します。

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup codex
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

`eidra setup <target> --write` を使うと、シェルや IDE を直接書き換えずに `~/.eidra/generated/<target>/` へ再利用可能なファイルを生成できます。
いまの named target は Cursor、Claude Code、Codex CLI、OpenAI / Anthropic 系 SDK、GitHub Actions、MCP 向けです。

## CI と自動化

`eidra scan --json`、`eidra doctor --json`、`eidra config validate --json` は、CI、スクリプト、他の AI ツールがそのまま消費しやすい出力です。人間向けテキストを後からパースする必要がありません。

## ポリシー例

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

## さらに詳しく

MCP Semantic RBAC、カスタムスキャンルール、セキュアチャネル、アーキテクチャ、ロードマップなどの最新詳細は [英語 README](../README.md) を参照してください。翻訳ページは「最初に入れやすいこと」と「主要コマンドがすぐ分かること」を優先して同期しています。

## コントリビュート

MIT ライセンスです。PR は歓迎です。詳細は [CONTRIBUTING.md](contributing.md) を参照してください。
