# Eidra のユースケース

Eidra を「なぜ入れるのか」を最短で理解するには、この3つを見るのがいちばん早いです。

## 1. コーディングエージェントが `.env` を送ろうとする

Claude Code、Codex、Cursor などでデバッグや修正を頼むと、広いファイル文脈が集められます。
その中に `.env`、API キー、DB 認証情報が入ることがあります。

Eidra がない場合:

- その文脈がそのままクラウド API に送られる
- 生の送信内容を自分で見づらい

Eidra がある場合:

- 端末の外へ出る前にリクエストをスキャンする
- policy に応じて秘密情報をマスクまたはブロックできる
- ダッシュボードと監査ログで何が起きたか追える

典型的な結果:

```text
.env の内容を検出 -> 外へ出る前に mask または block
```

## 2. MCP の tool call が危険になる

shell、filesystem、database 用の MCP server をつなぐと、AI は単なる会話ではなく「操作権限」も持ちます。

Eidra がない場合:

- `run_command("rm -rf /")`
- `execute_sql("DROP TABLE users")`
- `read_file("/etc/shadow")`

のような呼び出しが、そのまま届く可能性があります。

Eidra がある場合:

- 接続してよい MCP server を制限できる
- tool 単位で allow / block を決められる
- 引数の中身を見て破壊的パターンを止められる
- 戻り値側の機密情報もスキャンできる

典型的な結果:

```text
execute_sql("DROP TABLE users") -> MCP policy で block
```

## 3. クラウドに送りたくないリクエストがある

全部をクラウドに送りたいわけではありません。
機密度が高いものだけローカルで扱いたい場面があります。

Eidra がない場合:

- 速さと統制のどちらかを諦めがち

Eidra がある場合:

- 同じリクエストをローカルでスキャンできる
- policy で allow / mask / block / route を選べる
- 機密度の高い OpenAI 互換 chat リクエストを Ollama に逃がせる

典型的な結果:

```text
PII や社内コードを検出 -> リクエストを local model へ route
```

## 最初にやること

```bash
eidra init
eidra doctor
eidra setup codex
eidra dashboard
```

そのあと、まずは次のどれかを試す人が多いです。

- `eidra scan` にテスト用の secret を流す
- エディタや agent を proxy 経由で起動する
- `eidra gateway` を MCP の前段に置く

## 次に読むもの

- [What Is Eidra?](what-is-eidra.md)
- [For Developers](for-developers.md)
- [Architecture](architecture.md)
