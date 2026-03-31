use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    AiRequest,
    ScanFinding,
    PolicyAction,
    AgentMessage,
    IdentityVerification,
    Custom(String),
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AiRequest => write!(f, "ai_request"),
            Self::ScanFinding => write!(f, "scan_finding"),
            Self::PolicyAction => write!(f, "policy_action"),
            Self::AgentMessage => write!(f, "agent_message"),
            Self::IdentityVerification => write!(f, "identity_verification"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionTaken {
    Allow,
    Mask,
    Block,
    Escalate,
    Custom(String),
}

impl std::fmt::Display for ActionTaken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Mask => write!(f, "mask"),
            Self::Block => write!(f, "block"),
            Self::Escalate => write!(f, "escalate"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub action: ActionTaken,
    pub destination: String,
    pub findings_count: u32,
    pub findings_summary: String,
    pub data_size_bytes: u64,
    pub metadata: HashMap<String, String>,
}

impl AuditEvent {
    pub fn new(
        event_type: EventType,
        action: ActionTaken,
        destination: impl Into<String>,
        findings_count: u32,
        findings_summary: impl Into<String>,
        data_size_bytes: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            action,
            destination: destination.into(),
            findings_count,
            findings_summary: findings_summary.into(),
            data_size_bytes,
            metadata: HashMap::new(),
        }
    }
}
