use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("audit error: {0}")]
    Audit(#[from] eidra_audit::error::AuditError),

    #[error("invalid uri: {0}")]
    InvalidUri(String),

    #[error("connection failed: {0}")]
    ConnectionFailed(String),
}
