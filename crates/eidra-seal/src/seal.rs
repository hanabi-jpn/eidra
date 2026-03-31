use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use crate::entry::SealedMetadataEntry;
use crate::error::{Result, SealError};

/// Nonce size for AES-256-GCM (12 bytes).
const NONCE_SIZE: usize = 12;

/// Generate a random 256-bit key for sealing metadata.
///
/// In v1, this is a single key stored locally.
/// In v2, this key will be split via Shamir's Secret Sharing (2-of-2).
pub fn generate_seal_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Encrypt a metadata entry with AES-256-GCM.
///
/// Returns nonce (12 bytes) || ciphertext.
pub fn seal_entry(key: &[u8; 32], entry: &SealedMetadataEntry) -> Result<Vec<u8>> {
    let plaintext = serde_json::to_vec(entry)
        .map_err(|e| SealError::Serialization(format!("failed to serialize entry: {e}")))?;

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| SealError::Encryption(format!("failed to create cipher: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_slice())
        .map_err(|e| SealError::Encryption(format!("encryption failed: {e}")))?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt a sealed metadata entry with AES-256-GCM.
pub fn unseal_entry(key: &[u8; 32], sealed_data: &[u8]) -> Result<SealedMetadataEntry> {
    if sealed_data.len() < NONCE_SIZE {
        return Err(SealError::Decryption(
            "sealed data too short (missing nonce)".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = sealed_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| SealError::Decryption(format!("failed to create cipher: {e}")))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| SealError::Decryption(format!("decryption failed: {e}")))?;

    serde_json::from_slice(&plaintext)
        .map_err(|e| SealError::Serialization(format!("failed to deserialize entry: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{SealedMetadataEntry, SessionType};

    fn sample_entry() -> SealedMetadataEntry {
        let mut entry = SealedMetadataEntry::new(
            SessionType::AiRequest,
            "a3f2deadbeef",
            "api.anthropic.com",
            "mask",
        );
        entry.findings_count = 2;
        entry.findings_categories = vec!["api_key".into(), "pii".into()];
        entry.data_size_bytes = 8192;
        entry.policy_rule = "default:high-severity".into();
        entry
    }

    #[test]
    fn seal_unseal_roundtrip() {
        let key = generate_seal_key();
        let entry = sample_entry();

        let sealed = seal_entry(&key, &entry).expect("sealing should succeed");
        assert!(sealed.len() > NONCE_SIZE);

        let unsealed = unseal_entry(&key, &sealed).expect("unsealing should succeed");
        assert_eq!(unsealed.action, "mask");
        assert_eq!(unsealed.findings_count, 2);
        assert_eq!(unsealed.findings_categories, vec!["api_key", "pii"]);
        assert_eq!(unsealed.data_size_bytes, 8192);
        assert_eq!(unsealed.source_device_hash, "a3f2deadbeef");
        assert_eq!(unsealed.destination_hash, "api.anthropic.com");
        assert_eq!(unsealed.policy_rule, "default:high-severity");
    }

    #[test]
    fn unseal_with_wrong_key_fails() {
        let key = generate_seal_key();
        let wrong_key = generate_seal_key();
        let entry = sample_entry();

        let sealed = seal_entry(&key, &entry).expect("sealing should succeed");
        let result = unseal_entry(&wrong_key, &sealed);
        assert!(result.is_err());
    }

    #[test]
    fn unseal_too_short_data_fails() {
        let key = generate_seal_key();
        let result = unseal_entry(&key, &[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn all_session_types_serialize() {
        let key = generate_seal_key();

        for session_type in [
            SessionType::AiRequest,
            SessionType::SecureChannel,
            SessionType::IdentityVerification,
            SessionType::AgentMessage,
            SessionType::PaymentAuthorization,
            SessionType::Custom("test".into()),
        ] {
            let entry = SealedMetadataEntry::new(session_type, "src", "dst", "allow");
            let sealed = seal_entry(&key, &entry).expect("sealing should succeed");
            let unsealed = unseal_entry(&key, &sealed).expect("unsealing should succeed");
            assert_eq!(unsealed.action, "allow");
        }
    }
}
