use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The type of credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialType {
    /// Attestation that a request comes from a registered Eidra device.
    DeviceAttestation,
    /// Role credential for an AI agent (e.g., "EXECUTOR", "SENTINEL").
    AgentRole,
    /// Custom credential type.
    Custom(String),
}

/// A verifiable credential stored in the local credential wallet.
///
/// In v1, this is a simple structured record. In v2, this will conform
/// to W3C Verifiable Credentials 2.0 with cryptographic proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Unique credential identifier.
    pub id: String,
    /// The type of this credential.
    pub credential_type: CredentialType,
    /// Who issued this credential.
    pub issuer: String,
    /// Who this credential is about.
    pub subject: String,
    /// When the credential was issued.
    pub issued_at: DateTime<Utc>,
    /// When the credential expires (None = never).
    pub expires_at: Option<DateTime<Utc>>,
    /// Claims contained in this credential.
    pub claims: HashMap<String, String>,
    /// Extensible metadata.
    pub metadata: HashMap<String, String>,
}

impl Credential {
    /// Create a new credential.
    pub fn new(
        credential_type: CredentialType,
        issuer: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            credential_type,
            issuer: issuer.into(),
            subject: subject.into(),
            issued_at: Utc::now(),
            expires_at: None,
            claims: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

/// A local wallet for storing credentials.
///
/// In v1, this is in-memory only. In v2, credentials will be persisted
/// in an encrypted SQLite database.
#[derive(Debug, Default)]
pub struct CredentialWallet {
    credentials: Vec<Credential>,
}

impl CredentialWallet {
    /// Create a new empty credential wallet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a credential to the wallet.
    pub fn add_credential(&mut self, credential: Credential) {
        self.credentials.push(credential);
    }

    /// Find all credentials matching the given type.
    pub fn find_by_type(&self, credential_type: &CredentialType) -> Vec<&Credential> {
        self.credentials
            .iter()
            .filter(|c| &c.credential_type == credential_type)
            .collect()
    }

    /// Get the total number of credentials in the wallet.
    pub fn len(&self) -> usize {
        self.credentials.len()
    }

    /// Check if the wallet is empty.
    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_add_and_find() {
        let mut wallet = CredentialWallet::new();
        assert!(wallet.is_empty());

        let cred1 = Credential::new(
            CredentialType::DeviceAttestation,
            "eidra-local",
            "device-abc",
        );
        let cred2 = Credential::new(CredentialType::AgentRole, "eidra-local", "agent-executor");
        let cred3 = Credential::new(
            CredentialType::DeviceAttestation,
            "eidra-local",
            "device-xyz",
        );

        wallet.add_credential(cred1);
        wallet.add_credential(cred2);
        wallet.add_credential(cred3);

        assert_eq!(wallet.len(), 3);

        let attestations = wallet.find_by_type(&CredentialType::DeviceAttestation);
        assert_eq!(attestations.len(), 2);

        let roles = wallet.find_by_type(&CredentialType::AgentRole);
        assert_eq!(roles.len(), 1);

        let custom = wallet.find_by_type(&CredentialType::Custom("unknown".into()));
        assert!(custom.is_empty());
    }
}
