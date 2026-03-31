use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The type of session that generated this metadata entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    /// An AI model request (e.g., to Claude, GPT).
    AiRequest,
    /// An E2EE secure channel session.
    SecureChannel,
    /// A device/agent identity verification.
    IdentityVerification,
    /// An agent-to-agent message.
    AgentMessage,
    /// A payment authorization (future).
    PaymentAuthorization,
    /// Custom session type.
    Custom(String),
}

/// A sealed metadata entry as defined in CLAUDE.md Section 2.6.
///
/// Contains ONLY metadata about a session — never the content itself.
/// In v1, encrypted with a single local AES-256-GCM key.
/// In v2, the key will be split via Shamir's Secret Sharing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedMetadataEntry {
    /// When this event occurred.
    pub timestamp: DateTime<Utc>,
    /// The type of session.
    pub session_type: SessionType,
    /// SHA-256 hash of the source device's public key.
    pub source_device_hash: String,
    /// Destination identifier (API endpoint or peer device hash).
    pub destination_hash: String,
    /// Action taken: allow, mask, block, escalate.
    pub action: String,
    /// Number of findings from the scan engine.
    pub findings_count: u32,
    /// Categories of findings (e.g., "api_key", "pii") — NOT the actual values.
    pub findings_categories: Vec<String>,
    /// Size of the data in bytes.
    pub data_size_bytes: u64,
    /// The policy rule that triggered this action.
    pub policy_rule: String,
    /// Eidra version that produced this entry.
    pub eidra_version: String,
}

impl SealedMetadataEntry {
    /// Create a new metadata entry with the current timestamp.
    pub fn new(
        session_type: SessionType,
        source_device_hash: impl Into<String>,
        destination_hash: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            session_type,
            source_device_hash: source_device_hash.into(),
            destination_hash: destination_hash.into(),
            action: action.into(),
            findings_count: 0,
            findings_categories: Vec::new(),
            data_size_bytes: 0,
            policy_rule: String::new(),
            eidra_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
