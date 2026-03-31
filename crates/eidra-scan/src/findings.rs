use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
    Custom(String),
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
            Self::Info => write!(f, "INFO"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Category {
    ApiKey,
    SecretKey,
    PrivateKey,
    Token,
    Credential,
    Pii,
    InternalInfra,
    SensitivePath,
    HighEntropy,
    Custom(String),
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey => write!(f, "api_key"),
            Self::SecretKey => write!(f, "secret_key"),
            Self::PrivateKey => write!(f, "private_key"),
            Self::Token => write!(f, "token"),
            Self::Credential => write!(f, "credential"),
            Self::Pii => write!(f, "pii"),
            Self::InternalInfra => write!(f, "internal_infra"),
            Self::SensitivePath => write!(f, "sensitive_path"),
            Self::HighEntropy => write!(f, "high_entropy"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub category: Category,
    pub severity: Severity,
    pub rule_name: String,
    pub description: String,
    pub matched_text: String,
    pub offset: usize,
    pub length: usize,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Finding {
    pub fn new(
        category: Category,
        severity: Severity,
        rule_name: impl Into<String>,
        description: impl Into<String>,
        matched_text: impl Into<String>,
        offset: usize,
        length: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            severity,
            rule_name: rule_name.into(),
            description: description.into(),
            matched_text: matched_text.into(),
            offset,
            length,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
