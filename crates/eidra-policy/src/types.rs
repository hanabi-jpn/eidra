use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    Allow,
    Mask,
    Block,
    Escalate,
    Route(RouteTarget),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouteTarget {
    Local,
    Cloud,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "match")]
    pub match_conditions: MatchConditions,
    pub action: PolicyAction,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchConditions {
    #[serde(default)]
    pub severity: Option<MatchValue>,
    #[serde(default)]
    pub category: Option<MatchValue>,
    #[serde(default)]
    pub destination: Option<MatchValue>,
    #[serde(default)]
    pub rule_name: Option<MatchValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
    #[serde(default = "default_action")]
    pub default_action: PolicyAction,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_version() -> String {
    "1".to_string()
}

fn default_action() -> PolicyAction {
    PolicyAction::Allow
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            rules: Vec::new(),
            default_action: PolicyAction::Allow,
            metadata: HashMap::new(),
        }
    }
}

/// Context passed to the policy engine for evaluation.
pub struct PolicyContext<'a> {
    pub findings: &'a [eidra_scan::findings::Finding],
    pub destination: &'a str,
    pub data_size_bytes: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MatchValue {
    Single(String),
    Many(Vec<String>),
}

impl MatchValue {
    pub fn matches(&self, actual: &str) -> bool {
        match self {
            Self::Single(value) => matches_pattern(value, actual),
            Self::Many(values) => values.iter().any(|value| matches_pattern(value, actual)),
        }
    }
}

fn matches_pattern(expected: &str, actual: &str) -> bool {
    expected == "*" || expected.eq_ignore_ascii_case(actual)
}
