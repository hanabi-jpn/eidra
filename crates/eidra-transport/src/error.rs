use thiserror::Error;

/// Errors that can occur in the transport layer.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Cryptographic operation failed.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(String),

    /// The room has expired.
    #[error("room expired")]
    RoomExpired,

    /// The room was not found.
    #[error("room not found")]
    RoomNotFound,

    /// A peer disconnected unexpectedly.
    #[error("peer disconnected")]
    PeerDisconnected,

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, TransportError>;
