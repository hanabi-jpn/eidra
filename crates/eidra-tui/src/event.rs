use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEntry {
    pub timestamp: DateTime<Utc>,
    pub action: RequestAction,
    pub provider: String,
    pub findings_count: u32,
    pub categories: Vec<String>,
    pub data_size_bytes: u64,
    pub latency_ms: u64,
    pub status_code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestAction {
    Allow,
    Route,
    Mask,
    Block,
    Escalate,
}

impl std::fmt::Display for RequestAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "✓ ALLOW"),
            Self::Route => write!(f, "↺ ROUTE"),
            Self::Mask => write!(f, "◐ MASK"),
            Self::Block => write!(f, "✗ BLOCK"),
            Self::Escalate => write!(f, "⚡ ESCALATE"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub total_requests: u64,
    pub allowed: u64,
    pub routed: u64,
    pub masked: u64,
    pub blocked: u64,
    pub escalated: u64,
    pub total_findings: u64,
    pub categories: std::collections::HashMap<String, u64>,
}

impl Statistics {
    pub fn record(&mut self, entry: &RequestEntry) {
        self.total_requests += 1;
        self.total_findings += entry.findings_count as u64;
        match entry.action {
            RequestAction::Allow => self.allowed += 1,
            RequestAction::Route => self.routed += 1,
            RequestAction::Mask => self.masked += 1,
            RequestAction::Block => self.blocked += 1,
            RequestAction::Escalate => self.escalated += 1,
        }
        for cat in &entry.categories {
            *self.categories.entry(cat.clone()).or_insert(0) += 1;
        }
    }
}
