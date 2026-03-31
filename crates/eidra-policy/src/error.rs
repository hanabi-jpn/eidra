use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("yaml parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid policy: {0}")]
    Invalid(String),
}
