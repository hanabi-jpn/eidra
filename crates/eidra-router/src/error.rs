use thiserror::Error;

/// Errors that can occur in the router.
#[derive(Debug, Error)]
pub enum RouterError {
    /// The local Ollama instance is not available.
    #[error("Ollama is unavailable at endpoint '{endpoint}'")]
    OllamaUnavailable { endpoint: String },

    /// Error converting between API formats.
    #[error("Format conversion error: {0}")]
    FormatConversion(String),

    /// Error from the upstream LLM provider.
    #[error("Upstream error: {0}")]
    UpstreamError(String),

    /// HTTP/network transport error.
    #[error("Transport error: {0}")]
    Transport(#[from] hyper::Error),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Custom error for extensibility.
    #[error("Router error: {0}")]
    Custom(String),
}
