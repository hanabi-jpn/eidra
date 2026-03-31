use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand::RngCore;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use crate::error::{Result, TransportError};

/// Nonce size for XChaCha20-Poly1305 (24 bytes).
const NONCE_SIZE: usize = 24;

/// An X25519 key pair for E2EE key exchange.
pub struct KeyPair {
    /// The public key (safe to share).
    pub public_key: PublicKey,
    /// The secret key (never leaves the device).
    secret_key: StaticSecret,
}

impl KeyPair {
    /// Access the secret key (for key exchange).
    pub fn secret_key(&self) -> &StaticSecret {
        &self.secret_key
    }
}

// No manual Drop needed: StaticSecret is automatically zeroized on drop
// by x25519-dalek when the "zeroize" feature is enabled (which it is).

/// Generate a new X25519 key pair.
pub fn generate_keypair() -> KeyPair {
    let mut rng = rand::thread_rng();
    let secret_key = StaticSecret::random_from_rng(&mut rng);
    let public_key = PublicKey::from(&secret_key);
    KeyPair {
        public_key,
        secret_key,
    }
}

/// Derive a shared secret from our secret key and their public key (X25519 DH).
pub fn derive_shared_secret(our_secret: &StaticSecret, their_public: &PublicKey) -> SharedSecret {
    our_secret.diffie_hellman(their_public)
}

/// Encrypt plaintext using XChaCha20-Poly1305 with a 256-bit key.
///
/// Returns nonce (24 bytes) || ciphertext.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| TransportError::Crypto(format!("failed to create cipher: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| TransportError::Crypto(format!("encryption failed: {e}")))?;

    // Prepend nonce to ciphertext
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt ciphertext (nonce || ciphertext) using XChaCha20-Poly1305 with a 256-bit key.
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < NONCE_SIZE {
        return Err(TransportError::Crypto(
            "ciphertext too short (missing nonce)".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = XNonce::from_slice(nonce_bytes);

    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| TransportError::Crypto(format!("failed to create cipher: {e}")))?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| TransportError::Crypto(format!("decryption failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"Hello, Eidra! This is a secret message.";

        let encrypted = encrypt(&key, plaintext).expect("encryption should succeed");
        assert_ne!(&encrypted[NONCE_SIZE..], plaintext);

        let decrypted = decrypt(&key, &encrypted).expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong_key = [99u8; 32];
        let plaintext = b"secret data";

        let encrypted = encrypt(&key, plaintext).expect("encryption should succeed");
        let result = decrypt(&wrong_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn key_exchange_produces_shared_secret() {
        let alice = generate_keypair();
        let bob = generate_keypair();

        let alice_shared = derive_shared_secret(alice.secret_key(), &bob.public_key);
        let bob_shared = derive_shared_secret(bob.secret_key(), &alice.public_key);

        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }

    #[test]
    fn encrypt_decrypt_with_derived_key() {
        let alice = generate_keypair();
        let bob = generate_keypair();

        let shared = derive_shared_secret(alice.secret_key(), &bob.public_key);
        let key: [u8; 32] = *shared.as_bytes();

        let plaintext = b"agent-to-agent encrypted message";
        let encrypted = encrypt(&key, plaintext).expect("encryption should succeed");
        let decrypted = decrypt(&key, &encrypted).expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_too_short_data_fails() {
        let key = [0u8; 32];
        let result = decrypt(&key, &[1, 2, 3]);
        assert!(result.is_err());
    }
}
