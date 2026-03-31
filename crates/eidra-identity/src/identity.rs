use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{IdentityError, Result};

/// A device-bound identity. In v1, this uses a software key pair.
/// Future versions will use hardware secure elements (Secure Enclave, TPM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// SHA-256 hash of the public key, used as the device identifier.
    pub device_id: String,
    /// The public key bytes.
    pub public_key: Vec<u8>,
    /// When this identity was created.
    pub created_at: DateTime<Utc>,
    /// Extensible metadata.
    pub metadata: HashMap<String, String>,
}

impl DeviceIdentity {
    /// Generate a new device identity with a random 32-byte key pair.
    ///
    /// In v1 this is a software key pair. The device_id is the SHA-256 hash
    /// of the public key, providing a stable identifier without exposing the key.
    pub fn generate() -> Result<Self> {
        let mut rng = rand::thread_rng();

        // Generate a random 32-byte "secret key"
        let mut secret_key = [0u8; 32];
        rng.try_fill_bytes(&mut secret_key)
            .map_err(|e| IdentityError::KeyGeneration(format!("RNG failed: {e}")))?;

        // Derive "public key" by hashing the secret key (v1 simplification;
        // in v2 this would be a proper asymmetric derivation via Secure Enclave)
        let public_key = {
            let mut hasher = Sha256::new();
            hasher.update(b"eidra-v1-pubkey-derive:");
            hasher.update(secret_key);
            hasher.finalize().to_vec()
        };

        // device_id = SHA-256 of public key (hex encoded)
        let device_id = Self::compute_device_id(&public_key);

        Ok(Self {
            device_id,
            public_key,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Generate a device identity from a known public key.
    /// Useful for reconstructing identity from stored keys.
    pub fn from_public_key(public_key: Vec<u8>) -> Self {
        let device_id = Self::compute_device_id(&public_key);
        Self {
            device_id,
            public_key,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Get the device ID hash.
    pub fn device_id_hash(&self) -> &str {
        &self.device_id
    }

    /// Compute the device ID from a public key (SHA-256 hex).
    fn compute_device_id(public_key: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        hex::encode(hash)
    }
}

/// Simple hex encoding (no external dep needed).
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_valid_identity() {
        let identity = DeviceIdentity::generate().expect("should generate identity");
        assert!(!identity.device_id.is_empty());
        assert_eq!(identity.public_key.len(), 32);
        // device_id is SHA-256 hex = 64 chars
        assert_eq!(identity.device_id.len(), 64);
    }

    #[test]
    fn device_id_is_deterministic_from_same_key() {
        let public_key = vec![1u8; 32];
        let id1 = DeviceIdentity::from_public_key(public_key.clone());
        let id2 = DeviceIdentity::from_public_key(public_key);

        assert_eq!(id1.device_id, id2.device_id);
        assert_eq!(id1.device_id_hash(), id2.device_id_hash());
    }

    #[test]
    fn different_keys_produce_different_ids() {
        let id1 = DeviceIdentity::from_public_key(vec![1u8; 32]);
        let id2 = DeviceIdentity::from_public_key(vec![2u8; 32]);
        assert_ne!(id1.device_id, id2.device_id);
    }
}
