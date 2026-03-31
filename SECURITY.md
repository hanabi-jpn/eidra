# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Eidra, **please do not open a public issue.**

Instead, please report it privately:

- Email: security@eidra.dev
- Include: description, reproduction steps, and impact assessment
- We will acknowledge within 48 hours and provide a fix timeline within 7 days

## Scope

Eidra is a security tool. We take vulnerabilities in Eidra itself extremely seriously:

- **Critical**: Bypass of scan rules, policy engine, or masking that allows secrets to leak
- **High**: TLS/crypto implementation flaws, key material exposure
- **Medium**: Denial of service, resource exhaustion, information disclosure
- **Low**: Configuration issues, documentation errors

## Security Design

### What Eidra does
- Scans AI tool traffic for secrets and PII using regex rules (no ML, no cloud)
- Applies policy-based masking/blocking before data leaves your device
- All processing is local — Eidra never sends your data anywhere

### What Eidra does NOT do
- Eidra does not store your secrets (findings contain category and hash, not the actual secret)
- Eidra does not phone home or collect telemetry
- Eidra does not modify non-AI traffic (only AI provider domains are intercepted)

### HTTPS Interception
- Eidra performs TLS MITM **only** for known AI provider domains (api.openai.com, api.anthropic.com, etc.)
- Non-AI HTTPS connections are tunneled transparently without interception
- The local CA certificate is generated during `eidra init` and must be explicitly trusted by the user
- CA private key is stored with 0600 permissions at `~/.eidra/ca-key.pem`

### Cryptography
- E2EE channels: X25519 key exchange + ChaCha20-Poly1305 (AEAD)
- Sealed metadata: AES-256-GCM
- Key zeroization: Secret keys are zeroized on drop via the `zeroize` crate
- Session keys are ephemeral and never persisted to disk

### Known Limitations (v0.1)
- HTTPS MITM does not support HTTP/2 (HTTP/1.1 only)
- Chunked transfer encoding in MITM mode is not fully supported
- Split-key sealed metadata (Shamir's Secret Sharing) is not yet implemented
- Device identity uses software key pairs, not hardware secure elements

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Dependencies

Eidra uses well-audited cryptography crates:
- `x25519-dalek` (RustCrypto)
- `chacha20poly1305` (RustCrypto)
- `aes-gcm` (RustCrypto)
- `rustls` (no OpenSSL)
- `rcgen` (certificate generation)
