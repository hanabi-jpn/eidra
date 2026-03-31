use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::classifier::Classifier;
use crate::findings::{Category, Finding, Severity};

/// A custom rule defined in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRuleDefinition {
    pub name: String,
    pub pattern: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default)]
    pub description: String,
}

fn default_category() -> String {
    "custom".to_string()
}

fn default_severity() -> String {
    "medium".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRulesConfig {
    #[serde(default)]
    pub rules: Vec<CustomRuleDefinition>,
}

/// A classifier that uses custom YAML-defined rules.
pub struct CustomClassifier {
    rules: Vec<CompiledCustomRule>,
}

struct CompiledCustomRule {
    name: String,
    pattern: Regex,
    category: Category,
    severity: Severity,
    description: String,
}

impl CustomClassifier {
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        let config: CustomRulesConfig =
            serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {}", e))?;
        Self::from_config(config)
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("File read error: {}", e))?;
        Self::from_yaml(&content)
    }

    fn from_config(config: CustomRulesConfig) -> Result<Self, String> {
        let mut rules = Vec::new();
        for def in config.rules {
            let pattern = Regex::new(&def.pattern)
                .map_err(|e| format!("Invalid regex in rule '{}': {}", def.name, e))?;
            let category = parse_category(&def.category);
            let severity = parse_severity(&def.severity);
            rules.push(CompiledCustomRule {
                name: def.name,
                pattern,
                category,
                severity,
                description: def.description,
            });
        }
        Ok(Self { rules })
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Classifier for CustomClassifier {
    fn classify(&self, input: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            for mat in rule.pattern.find_iter(input) {
                findings.push(Finding::new(
                    rule.category.clone(),
                    rule.severity.clone(),
                    &rule.name,
                    &rule.description,
                    mat.as_str(),
                    mat.start(),
                    mat.len(),
                ));
            }
        }
        findings
    }

    fn name(&self) -> &str {
        "custom_classifier"
    }
}

fn parse_category(s: &str) -> Category {
    match s.to_lowercase().as_str() {
        "api_key" => Category::ApiKey,
        "secret_key" => Category::SecretKey,
        "private_key" => Category::PrivateKey,
        "token" => Category::Token,
        "credential" => Category::Credential,
        "pii" => Category::Pii,
        "internal_infra" => Category::InternalInfra,
        "sensitive_path" => Category::SensitivePath,
        "high_entropy" => Category::HighEntropy,
        other => Category::Custom(other.to_string()),
    }
}

fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        "info" => Severity::Info,
        other => Severity::Custom(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_rule_from_yaml() {
        let yaml = r#"
rules:
  - name: internal_project_id
    pattern: "PROJ-[0-9]{6}"
    category: internal_infra
    severity: medium
    description: "Internal project identifier"
  - name: company_email
    pattern: "[a-z]+@mycompany\\.com"
    category: pii
    severity: low
    description: "Company email address"
"#;
        let classifier = CustomClassifier::from_yaml(yaml).unwrap();
        assert_eq!(classifier.rule_count(), 2);

        let findings = classifier.classify("ticket PROJ-123456 by alice@mycompany.com");
        assert_eq!(findings.len(), 2);
        assert!(findings
            .iter()
            .any(|f| f.rule_name == "internal_project_id"));
        assert!(findings.iter().any(|f| f.rule_name == "company_email"));
    }

    #[test]
    fn test_custom_rule_no_match() {
        let yaml = r#"
rules:
  - name: test_rule
    pattern: "SECRET_[A-Z]{10}"
    category: secret_key
    severity: high
    description: "Test secret"
"#;
        let classifier = CustomClassifier::from_yaml(yaml).unwrap();
        let findings = classifier.classify("nothing sensitive here");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_invalid_regex() {
        let yaml = r#"
rules:
  - name: bad_rule
    pattern: "[invalid"
    category: custom
    severity: medium
    description: "Bad regex"
"#;
        let result = CustomClassifier::from_yaml(yaml);
        assert!(result.is_err());
    }
}
