use thiserror::Error;

/// Errors that can occur in the sealed metadata layer.
#[derive(Debug, Error)]
pub enum SealError {
    /// Encryption failed.
    #[error("encryption error: {0}")]
    Encryption(String),

    /// Decryption failed.
    #[error("decryption error: {0}")]
    Decryption(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, SealError>;
