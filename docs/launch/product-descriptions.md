# Eidra — 製品説明（ターゲット別 3層）

---

## Layer 1: 一般向け（AI触り始めた人、非エンジニア、経営者）

### ワンライナー
**「AIに送っているデータ、全部見えてますか？」**

### エレベーターピッチ（30秒）
ChatGPTやCursor、CopilotなどのAIツールを使うとき、あなたのパスワード、API鍵、顧客情報が知らないうちにクラウドに送られています。

Eidra（エイドラ）は、あなたのパソコンの中で動く「AIの番人」です。AIに送られるすべてのデータをリアルタイムでチェックし、危険な情報は自動的に隠してから送信します。AIの便利さはそのまま、情報漏洩だけを止めます。

### たとえ話
> 会社の受付に「来客チェック担当」がいるように、Eidraはあなたのパソコンから出ていくすべてのAI通信の「出口チェック担当」です。
>
> - 社員証（APIキー）を持ち出そうとしている → 止める
> - 顧客名簿（個人情報）を持ち出そうとしている → 黒塗りにして渡す
> - 普通の書類（一般的なコード）→ そのまま通す
>
> しかも、この担当者はあなたのパソコンの中にいるので、外部に情報が漏れる心配は一切ありません。

### ビジュアル説明
```
あなた → [Eidra 🛡️ チェック] → AI (ChatGPT, Cursor等)

✓ 普通の質問    → そのまま送信
◐ パスワード入り → パスワードだけ隠して送信
✗ 秘密鍵       → 送信をブロック
```

### キーメッセージ
- インストールは3行のコマンドだけ
- 設定不要、すぐ使える
- すべてあなたのパソコン内で処理（外部サーバーに何も送らない）
- オープンソース（無料、誰でもコードを確認できる）

---

## Layer 2: 開発者向け（エンジニア、AIツールのヘビーユーザー）

### ワンライナー
**「Claude Codeがあなたの.envを読んでいる。Eidraはそれを止める。」**

### 何をするツールか（1分）
Eidraは、あなたのマシンで動くローカルプロキシ（Rust製、5ms以下のオーバーヘッド）です。AI APIへのHTTP/HTTPSリクエストを透過的にインターセプトし、47種類の正規表現ルールでシークレット・PII・認証情報をスキャンします。

検出したら、YAMLポリシーに従って：
- **Mask** — `AKIAIOSFODNN7EXAMPLE` → `[REDACTED:api_key:a3f2]` に置換して送信
- **Block** — SSH秘密鍵など → 403で即ブロック
- **Route** — 機密データを含む場合 → ローカルLLM（Ollama）に転送

さらに、MCP Semantic RBAC（意味論ベースのアクセス制御）を搭載：
- AIが`DROP TABLE`を実行しようとした → **ブロック**
- AIが`~/.ssh/id_rsa`を読もうとした → **ブロック**
- AIが`rm -rf /`を実行しようとした → **ブロック**
- AIが`SELECT * FROM users`を実行 → **許可**

リアルタイムTUIダッシュボードで、何が送られ、何がブロックされたかを一目で確認できます。

### 技術的な差別化
```
従来のシークレットスキャナー（gitleaks, truffleHog等）:
  → コミット後にスキャン（事後検出）
  → CIで検知しても、すでにAPIに送信済み

Eidra:
  → 送信前にインターセプト（事前防御）
  → データがマシンを離れる前に隠す
  → リアルタイムで可視化
```

### クイックスタート
```bash
curl -sf eidra.dev/install | sh
eidra init        # CA証明書生成、設定ファイル作成
eidra dashboard   # プロキシ起動 + TUIダッシュボード

# 別ターミナルで
echo "AKIAIOSFODNN7EXAMPLE password=hunter2" | eidra scan
```

### 主要機能
| 機能 | 説明 |
|---|---|
| 47スキャンルール | AWS, GitHub, Slack, Stripe, JWT, PII, マイナンバー, etc. |
| YAMLポリシー | severity/category/destinationベースのmask/block/route |
| HTTPS MITM | AI providerドメインのみTLS intercept（hyper native） |
| MCP Semantic RBAC | ツール引数のregexマッチングでDROP/rm -rf等をブロック |
| Ollamaルーティング | 機密データ → ローカルLLMに自動転送 |
| TUIダッシュボード | ratatui製、リアルタイムリクエストストリーム |
| E2EEチャット | X25519 + XChaCha20-Poly1305、SAS認証 |
| カスタムルール | YAML定義の正規表現ルール追加 |
| 監査ログ | SQLiteに全アクション記録 |

---

## Layer 3: 技術者・投資家・アーキテクト向け（深掘り構造説明）

### ワンライナー
**「Edge-native zero-trust firewall for autonomous AI agents and MCP tools.」**

### アーキテクチャ概要

```
┌─────────────────────────────────────────────┐
│                  Eidra                       │
│                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │eidra-scan│  │eidra-    │  │eidra-    │  │
│  │47 regex  │→ │policy    │→ │router    │  │
│  │rules     │  │YAML eval │  │mask/block│  │
│  │          │  │          │  │/route    │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│       ↑                           ↓         │
│  ┌──────────┐              ┌──────────┐    │
│  │eidra-    │              │eidra-    │    │
│  │proxy     │              │tui       │    │
│  │HTTP/HTTPS│              │ratatui   │    │
│  │MITM      │              │dashboard │    │
│  └──────────┘              └──────────┘    │
│       ↑                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │eidra-mcp │  │eidra-    │  │eidra-    │  │
│  │Semantic  │  │transport │  │identity  │  │
│  │RBAC      │  │XChaCha20 │  │Device ID │  │
│  │firewall  │  │E2EE      │  │          │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│                                             │
│  ┌──────────┐  ┌──────────┐                │
│  │eidra-    │  │eidra-    │                │
│  │audit     │  │seal      │                │
│  │SQLite    │  │AES-256   │                │
│  │          │  │metadata  │                │
│  └──────────┘  └──────────┘                │
└─────────────────────────────────────────────┘
```

### 11 Crateの責務

| Crate | 責務 | LOC | テスト数 |
|---|---|---|---|
| `eidra-scan` | データ分類エンジン。Classifier traitで拡張可能。47 built-in regex + YAML custom rules | ~1,400 | 97 |
| `eidra-policy` | YAMLポリシー評価。first-match wins、severity最大値でoverall action決定 | ~360 | 6 |
| `eidra-router` | マスキング（JSON-aware tree-walk）+ Ollamaルーティング（OpenAI→Ollama形式変換） | ~430 | 13 |
| `eidra-proxy` | HTTP/HTTPS MITMプロキシ。hyper 1.x native。AI providerドメインのみTLS intercept | ~970 | 2 |
| `eidra-mcp` | MCP Gateway。JSON-RPC proxy + Semantic RBAC（ツール引数のregexマッチング） | ~820 | 23 |
| `eidra-audit` | SQLite監査ログ。全request/action/findingを記録 | ~300 | 2 |
| `eidra-transport` | E2EE。X25519 ECDH + XChaCha20-Poly1305。Room管理 | ~330 | 4 |
| `eidra-identity` | デバイスバウンドID。SHA-256鍵ハッシュ + Credential Wallet | ~290 | 4 |
| `eidra-seal` | Sealed Metadata。AES-256-GCM暗号化。v2でShamir split-key | ~240 | 8 |
| `eidra-tui` | ratatui TUI。ライブストリーム + 統計パネル + ヘルプバー | ~420 | — |
| `eidra-core` | CLIバイナリ。clap。init/start/stop/scan/dashboard/escape/join/config | ~730 | — |

### セキュリティ設計

**データフロー:**
```
Client Request → [Proxy Intercept]
                       ↓
                [Scan: 47 regex rules]
                       ↓
                [Policy: YAML evaluation]
                       ↓
              ┌────────┼────────┐
              ↓        ↓        ↓
           [ALLOW]  [MASK]   [BLOCK]
           forward  redact   403 JSON
           as-is    & send   response
```

**信頼モデル:**
- Content（メッセージ、コード、プロンプト）: E2EE。Eidra自身も読めない
- Metadata（誰が、いつ、サイズ、アクション）: AES-256-GCM暗号化
- すべてオンデバイス。Eidraは一切の外部通信をしない

**暗号スタック:**
- TLS: rustls 0.23（OpenSSL不使用）
- MITM証明書: rcgen 0.13で動的生成、ドメインごとにキャッシュ
- E2EE: X25519鍵交換 + XChaCha20-Poly1305（192-bit nonce）
- Sealed: AES-256-GCM
- 鍵ゼロ化: x25519-dalek zeroize feature

**MCP Semantic RBAC:**
```yaml
tool_rules:
  - tool: "execute_sql"
    block_patterns: ["(?i)\\b(DROP|DELETE|TRUNCATE)\\b"]
    # AIが "DROP TABLE users" を実行 → BLOCKED
    # AIが "SELECT * FROM users" を実行 → ALLOWED

  - tool: "read_file"
    blocked_paths: ["~/.ssh/**", "~/.aws/**", "**/.env"]
    # AIが ~/.ssh/id_rsa を読む → BLOCKED
    # AIが ~/code/main.rs を読む → ALLOWED

  - tool: "run_command"
    block_patterns: ["rm\\s+(-rf|--recursive)", "curl.*\\|\\s*sh"]
    # AIが "rm -rf /" を実行 → BLOCKED
    # AIが "ls -la" を実行 → ALLOWED
```

### ロードマップ

**v0.1 (current):** Data scanner + Policy engine + MCP Semantic RBAC + TUI + E2EE

**v0.2:** Local SLM intent scanning — オンデバイスの小型言語モデルが「この行動は悪意があるか？」をリアルタイム判定。AI対AIの防衛。

**v0.3:** Agent trust mesh — デバイスバウンドIDによるエージェント間相互認証。Sealed metadata with Shamir's Secret Sharing。

### なぜ今か

1. Claude Codeが`.env`を無断で読む（[報告済み](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage)）
2. Copilot使用リポジトリのシークレット漏洩率が40%高い
3. MCPツールにCVSS 8.6+の脆弱性（[IDEsaster](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html)）
4. 自律型AIエージェントが`rm -rf`や`DROP TABLE`を実行できる権限を持ち始めている
5. これらを止めるツールが存在しない

---

## SNS/DM用ショートバージョン

### Twitter/X用（280文字以内）
```
Your AI tools are leaking your secrets.

Eidra scans every AI request, masks API keys before they leave your machine, and blocks destructive MCP tool calls (DROP TABLE, rm -rf).

47 rules. Zero config. Everything on-device. Rust + MIT.

🔗 github.com/hanabi-jpn/eidra
```

### DM用（研究者/インフルエンサー向け・英語）
```
Hi [name],

I built an open-source tool that might interest you — Eidra is a local proxy (Rust) that intercepts AI tool traffic and scans for secrets before they reach cloud APIs.

Key features:
- 47 scan rules (API keys, PII, private keys)
- MCP Semantic RBAC — blocks "DROP TABLE" but allows "SELECT"
- HTTPS MITM for AI providers (hyper native, not manual parsing)
- TUI dashboard showing everything in real-time

I'm launching on HN next week. Would you be open to trying it and sharing feedback? I can give you early access to the private repo.

Link: [private repo URL]
```

### DM用（日本語・AI系発信者向け）
```
[name]さん

AIセキュリティのOSSを開発しました。Eidra（エイドラ）というツールで、CursorやClaude CodeなどのAIツールが送信するデータをリアルタイムでスキャンし、APIキーや個人情報を自動的にマスク/ブロックします。

特徴的な機能として「MCP Semantic RBAC」があり、AIエージェントが`DROP TABLE`や`rm -rf`を実行しようとした瞬間にブロックします。

来週HNとGitHubで公開予定です。もしよければ事前に触ってフィードバックいただけないでしょうか？

47スキャンルール、161テスト、Rust製、MITライセンスです。
```

### DM用（日本語・一般AI系発信者向け）
```
[name]さん

突然のDM失礼します。AIツールのセキュリティに関するOSSを開発しました。

ChatGPTやCursor等のAIツールを使うとき、パスワードやAPI鍵が気づかないうちにクラウドに送られている問題をご存知ですか？

Eidra（エイドラ）は、パソコンの中で動く「AIの番人」です。AIに送られるデータをリアルタイムで監視し、危険な情報だけ自動的に隠します。インストールは3行、設定不要です。

来週公開予定で、もしご興味あれば事前にお見せできればと思います。
```
