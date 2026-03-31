use crate::types::{MatchConditions, PolicyAction, PolicyConfig, PolicyContext};
use eidra_scan::findings::Finding;

/// Result of policy evaluation for a single finding.
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub finding: Finding,
    pub action: PolicyAction,
    pub matched_rule: Option<String>,
}

/// Result of evaluating all findings in a request.
#[derive(Debug)]
pub struct RequestDecision {
    pub overall_action: PolicyAction,
    pub decisions: Vec<PolicyDecision>,
}

pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    /// Evaluate a single finding against the policy rules.
    /// First match wins (top-to-bottom).
    pub fn evaluate_finding(&self, finding: &Finding, destination: &str) -> PolicyDecision {
        for rule in &self.config.rules {
            if matches_conditions(&rule.match_conditions, finding, destination) {
                return PolicyDecision {
                    finding: finding.clone(),
                    action: rule.action.clone(),
                    matched_rule: Some(rule.name.clone()),
                };
            }
        }
        PolicyDecision {
            finding: finding.clone(),
            action: self.config.default_action.clone(),
            matched_rule: None,
        }
    }

    /// Evaluate all findings for a request and determine the overall action.
    /// The most restrictive action wins: Block > Mask > Escalate > Route > Allow.
    pub fn evaluate(&self, ctx: &PolicyContext<'_>) -> RequestDecision {
        let decisions: Vec<PolicyDecision> = ctx
            .findings
            .iter()
            .map(|f| self.evaluate_finding(f, ctx.destination))
            .collect();

        let overall_action = decisions
            .iter()
            .map(|d| &d.action)
            .max_by_key(|a| action_severity(a))
            .cloned()
            .unwrap_or_else(|| self.config.default_action.clone());

        RequestDecision {
            overall_action,
            decisions,
        }
    }
}

fn action_severity(action: &PolicyAction) -> u8 {
    match action {
        PolicyAction::Allow => 0,
        PolicyAction::Route(_) => 1,
        PolicyAction::Escalate => 2,
        PolicyAction::Mask => 3,
        PolicyAction::Block => 4,
        PolicyAction::Custom(_) => 2,
    }
}

fn matches_conditions(conditions: &MatchConditions, finding: &Finding, destination: &str) -> bool {
    if let Some(ref sev) = conditions.severity {
        let finding_sev = finding.severity.to_string();
        if !sev.matches(&finding_sev) {
            return false;
        }
    }
    if let Some(ref cat) = conditions.category {
        let finding_cat = finding.category.to_string();
        if !cat.matches(&finding_cat) {
            return false;
        }
    }
    if let Some(ref dest) = conditions.destination {
        let is_local = destination == "localhost"
            || destination.starts_with("127.")
            || destination.starts_with("local");
        let dest_type = if is_local { "local" } else { "cloud" };
        if !dest.matches(dest_type) {
            return false;
        }
    }
    if let Some(ref rn) = conditions.rule_name {
        if !rn.matches(&finding.rule_name) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::default_policy;
    use eidra_scan::findings::{Category, Finding, Severity};

    fn make_finding(category: Category, severity: Severity, rule_name: &str) -> Finding {
        Finding::new(category, severity, rule_name, "test", "matched", 0, 7)
    }

    #[test]
    fn test_block_private_key() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::PrivateKey, Severity::Critical, "private_key");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Block);
    }

    #[test]
    fn test_mask_api_key() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::ApiKey, Severity::High, "aws_access_key");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Mask);
    }

    #[test]
    fn test_mask_pii_cloud() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::Pii, Severity::Medium, "email_address");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Mask);
    }

    #[test]
    fn test_allow_pii_local() {
        let engine = PolicyEngine::new(default_policy());
        let finding = make_finding(Category::Pii, Severity::Medium, "email_address");
        let decision = engine.evaluate_finding(&finding, "localhost:11434");
        assert_eq!(decision.action, PolicyAction::Allow);
    }

    #[test]
    fn test_allow_clean_request() {
        let engine = PolicyEngine::new(default_policy());
        let ctx = PolicyContext {
            findings: &[],
            destination: "api.openai.com",
            data_size_bytes: 100,
            metadata: std::collections::HashMap::new(),
        };
        let decision = engine.evaluate(&ctx);
        assert_eq!(decision.overall_action, PolicyAction::Allow);
    }

    #[test]
    fn test_overall_most_restrictive() {
        let engine = PolicyEngine::new(default_policy());
        let findings = vec![
            make_finding(Category::ApiKey, Severity::High, "aws_access_key"), // → mask
            make_finding(Category::PrivateKey, Severity::Critical, "private_key"), // → block
        ];
        let ctx = PolicyContext {
            findings: &findings,
            destination: "api.openai.com",
            data_size_bytes: 500,
            metadata: std::collections::HashMap::new(),
        };
        let decision = engine.evaluate(&ctx);
        assert_eq!(decision.overall_action, PolicyAction::Block);
    }

    #[test]
    fn test_match_conditions_support_lists() {
        let config = crate::loader::load_from_str(
            r#"
version: "1"
rules:
  - name: mask_sensitive
    match:
      category: ["api_key", "token"]
      destination: "cloud"
    action: mask
"#,
        )
        .expect("policy should parse");

        let engine = PolicyEngine::new(config);
        let finding = make_finding(Category::Token, Severity::High, "github_token");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Mask);
    }

    #[test]
    fn test_match_conditions_support_wildcards() {
        let config = crate::loader::load_from_str(
            r#"
version: "1"
rules:
  - name: allow_everything
    match:
      category: "*"
    action: allow
"#,
        )
        .expect("policy should parse");

        let engine = PolicyEngine::new(config);
        let finding = make_finding(Category::InternalInfra, Severity::Low, "corp_host");
        let decision = engine.evaluate_finding(&finding, "api.openai.com");
        assert_eq!(decision.action, PolicyAction::Allow);
    }
}
