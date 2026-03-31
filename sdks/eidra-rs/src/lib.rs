//! # Eidra SDK
//!
//! Re-exports core Eidra crates for easy integration into Rust projects.
//!
//! ```rust,no_run
//! use eidra_rs::scan::scanner::Scanner;
//!
//! let scanner = Scanner::with_defaults();
//! let findings = scanner.scan("my secret AKIAIOSFODNN7EXAMPLE");
//! println!("{} findings", findings.len());
//! ```

/// Data classification engine — scan text for secrets, PII, and sensitive data.
pub use eidra_scan as scan;

/// Policy evaluation engine — YAML-based rules for mask/block/allow/route.
pub use eidra_policy as policy;

/// E2EE transport — X25519 key exchange + ChaCha20-Poly1305 encryption.
pub use eidra_transport as transport;

/// Device-bound identity — key generation and credential wallet.
pub use eidra_identity as identity;
