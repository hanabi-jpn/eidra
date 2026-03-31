use crate::error::PolicyError;
use crate::types::PolicyConfig;
use std::path::Path;

pub fn load_from_str(yaml: &str) -> Result<PolicyConfig, PolicyError> {
    let config: PolicyConfig = serde_yaml::from_str(yaml)?;
    Ok(config)
}

pub fn load_from_file(path: &Path) -> Result<PolicyConfig, PolicyError> {
    let content = std::fs::read_to_string(path)?;
    load_from_str(&content)
}

pub fn default_policy() -> PolicyConfig {
    let yaml = include_str!("../../../config/policies/default.yaml");
    load_from_str(yaml).expect("default policy must be valid")
}
