use thiserror::Error;

/// Errors that can occur in the identity layer.
#[derive(Debug, Error)]
pub enum IdentityError {
    /// Key generation failed.
    #[error("key generation error: {0}")]
    KeyGeneration(String),

    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(String),

    /// Verification failed.
    #[error("verification error: {0}")]
    Verification(String),

    /// Identity not found.
    #[error("identity not found")]
    NotFound,

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, IdentityError>;
