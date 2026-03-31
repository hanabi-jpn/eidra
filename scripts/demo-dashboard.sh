#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

cleanup() {
  if [[ -n "${TRAFFIC_PID:-}" ]]; then
    kill "$TRAFFIC_PID" 2>/dev/null || true
  fi
}

trap cleanup EXIT

cargo build --quiet >/dev/null 2>&1
cargo run --quiet -p eidra-core -- init >/dev/null 2>&1 || true

send_openai() {
  local content="$1"
  curl -s -o /dev/null -x http://127.0.0.1:8080 \
    -X POST http://api.openai.com/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"${content}\"}]}" \
    || true
}

send_anthropic() {
  local content="$1"
  curl -s -o /dev/null -x http://127.0.0.1:8080 \
    -X POST http://api.anthropic.com/v1/messages \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"claude-sonnet-4\",\"max_tokens\":64,\"messages\":[{\"role\":\"user\",\"content\":\"${content}\"}]}" \
    || true
}

(
  sleep 2
  send_openai "What is the safest way to rotate staging credentials?"
  sleep 0.8
  send_openai "Deploy with AKIAIOSFODNN7EXAMPLE to the staging bucket"
  sleep 0.8
  send_anthropic "Contact john@acme.com and verify customer SSN 123-45-6789"
  sleep 0.8
  send_openai "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA-demo"
  sleep 0.8
  send_anthropic "Summarize security posture for local MCP tools"
  sleep 0.8
  send_openai "postgres://admin:secret@db.internal.corp/prod password=hunter2"
) &
TRAFFIC_PID=$!

RUST_LOG=error cargo run --quiet -p eidra-core -- dashboard
