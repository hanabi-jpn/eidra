#!/bin/bash
set -e

# Eidra demo script
# Starts the proxy and sends test requests to demonstrate scanning

echo "=== Eidra Demo ==="
echo ""

# Build first
echo "Building Eidra..."
cargo build --quiet 2>/dev/null

# Start proxy in background
echo "Starting proxy on localhost:8080..."
RUST_LOG=info cargo run --quiet -- start &
PROXY_PID=$!
sleep 2

echo ""
echo "=== Test 1: Clean request (should ALLOW) ==="
curl -s -x http://localhost:8080 \
  -X POST http://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"What is the capital of France?"}]}' \
  2>&1 || true

echo ""
echo "=== Test 2: AWS key in request (should MASK) ==="
curl -s -x http://localhost:8080 \
  -X POST http://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Deploy with AKIAIOSFODNN7EXAMPLE"}]}' \
  2>&1 || true

echo ""
echo "=== Test 3: Private key in request (should BLOCK) ==="
curl -s -x http://localhost:8080 \
  -X POST http://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"-----BEGIN RSA PRIVATE KEY-----\nMIIE..."}]}' \
  2>&1 || true

echo ""
echo "=== Test 4: PII in request (should MASK) ==="
curl -s -x http://localhost:8080 \
  -X POST http://api.anthropic.com/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Send email to john@acme.com, SSN 123-45-6789"}]}' \
  2>&1 || true

echo ""
echo "=== Standalone scan ==="
echo 'my AWS key AKIAIOSFODNN7EXAMPLE and password="hunter2" at postgres://admin:secret@db.internal.corp/prod' | cargo run --quiet -- scan

echo ""
echo "=== Demo complete ==="

# Cleanup
kill $PROXY_PID 2>/dev/null || true
