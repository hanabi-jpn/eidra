use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use eidra_audit::store::AuditStore;
use eidra_mcp::config::{McpGatewayConfig, McpServerEntry};
use eidra_policy::types::PolicyConfig;
use eidra_router::ollama::OllamaRouter;
use eidra_scan::rules::custom::CustomClassifier;
use eidra_scan::scanner::Scanner;

const DEFAULT_PROXY_LISTEN: &str = "127.0.0.1:8080";
const DEFAULT_PROXY_MAX_BODY_SIZE: usize = 10 * 1024 * 1024;
const DEFAULT_MCP_LISTEN: &str = "127.0.0.1:8081";
const DEFAULT_LOCAL_LLM_ENDPOINT: &str = "http://localhost:11434";
const DEFAULT_LOCAL_LLM_MODEL: &str = "qwen2.5:latest";
const DEFAULT_AUDIT_DB_PATH: &str = "~/.eidra/audit.db";

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub eidra_dir: PathBuf,
    pub config_path: PathBuf,
    pub policy_path: PathBuf,
    pub ca_cert_path: PathBuf,
    pub ca_key_path: PathBuf,
    pub pid_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub proxy: ProxySection,
    #[serde(default)]
    pub scan: ScanSection,
    #[serde(default)]
    pub local_llm: LocalLlmSection,
    #[serde(default)]
    pub mcp_gateway: McpGatewaySection,
    #[serde(default)]
    pub audit: AuditSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProxySection {
    #[serde(default = "default_proxy_listen")]
    pub listen: String,
    #[serde(default = "default_proxy_max_body_size")]
    pub max_body_size: usize,
}

impl Default for ProxySection {
    fn default() -> Self {
        Self {
            listen: default_proxy_listen(),
            max_body_size: default_proxy_max_body_size(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub custom_rules_path: String,
}

impl Default for ScanSection {
    fn default() -> Self {
        Self {
            enabled: true,
            custom_rules_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalLlmSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_local_llm_provider")]
    pub provider: String,
    #[serde(default = "default_local_llm_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_model_mapping")]
    pub model_mapping: HashMap<String, String>,
}

impl Default for LocalLlmSection {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_local_llm_provider(),
            endpoint: default_local_llm_endpoint(),
            model_mapping: default_model_mapping(),
        }
    }
}

impl LocalLlmSection {
    pub fn resolve_model(&self, requested_model: Option<&str>) -> &str {
        requested_model
            .and_then(|model| self.model_mapping.get(model))
            .or_else(|| self.model_mapping.get("default"))
            .map(String::as_str)
            .unwrap_or(DEFAULT_LOCAL_LLM_MODEL)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct McpGatewaySection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_mcp_listen")]
    pub listen: String,
    #[serde(default)]
    pub server_whitelist: HashMap<String, McpServerEntry>,
    #[serde(default)]
    pub servers: Vec<McpServerEntry>,
    #[serde(default)]
    pub global_rate_limit: u32,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for McpGatewaySection {
    fn default() -> Self {
        Self {
            enabled: false,
            listen: default_mcp_listen(),
            server_whitelist: HashMap::new(),
            servers: Vec::new(),
            global_rate_limit: 0,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_audit_db_path")]
    pub db_path: String,
}

impl Default for AuditSection {
    fn default() -> Self {
        Self {
            enabled: true,
            db_path: default_audit_db_path(),
        }
    }
}

pub fn runtime_paths() -> Result<RuntimePaths> {
    let home = home_dir()?;
    let eidra_dir = home.join(".eidra");

    Ok(RuntimePaths {
        config_path: eidra_dir.join("config.yaml"),
        policy_path: eidra_dir.join("policy.yaml"),
        ca_cert_path: eidra_dir.join("ca.pem"),
        ca_key_path: eidra_dir.join("ca-key.pem"),
        pid_path: eidra_dir.join("proxy.pid"),
        eidra_dir,
    })
}

pub fn load_app_config(paths: &RuntimePaths) -> Result<AppConfig> {
    if !paths.config_path.exists() {
        return Ok(AppConfig::default());
    }

    let content = std::fs::read_to_string(&paths.config_path).with_context(|| {
        format!(
            "failed to read configuration file at {}",
            paths.config_path.display()
        )
    })?;

    serde_yaml::from_str(&content).with_context(|| {
        format!(
            "failed to parse configuration file at {}",
            paths.config_path.display()
        )
    })
}

pub fn load_policy_config(paths: &RuntimePaths) -> Result<PolicyConfig> {
    if !paths.policy_path.exists() {
        return Ok(eidra_policy::loader::default_policy());
    }

    eidra_policy::loader::load_from_file(&paths.policy_path).with_context(|| {
        format!(
            "failed to parse policy file at {}",
            paths.policy_path.display()
        )
    })
}

pub fn build_scanner(config: &AppConfig, force_builtin_rules: bool) -> Result<Scanner> {
    let mut scanner = if config.scan.enabled || force_builtin_rules {
        Scanner::with_defaults()
    } else {
        Scanner::new()
    };

    if config.scan.custom_rules_path.trim().is_empty() {
        return Ok(scanner);
    }

    let custom_rules_path = expand_tilde(&config.scan.custom_rules_path)?;
    let classifier = CustomClassifier::from_file(&custom_rules_path)
        .map_err(anyhow::Error::msg)
        .with_context(|| {
            format!(
                "failed to load custom scan rules from {}",
                custom_rules_path.display()
            )
        })?;
    scanner.add_classifier(Box::new(classifier));
    Ok(scanner)
}

pub fn build_local_router(config: &AppConfig) -> Result<Option<OllamaRouter>> {
    if !config.local_llm.enabled {
        return Ok(None);
    }

    if !config.local_llm.provider.eq_ignore_ascii_case("ollama") {
        anyhow::bail!(
            "unsupported local LLM provider '{}' (currently only 'ollama' is supported)",
            config.local_llm.provider
        );
    }

    Ok(Some(OllamaRouter::with_model_mapping(
        &config.local_llm.endpoint,
        config.local_llm.resolve_model(None),
        config.local_llm.model_mapping.clone(),
    )))
}

pub fn build_mcp_gateway_config(config: &AppConfig) -> McpGatewayConfig {
    let mut server_whitelist = config.mcp_gateway.server_whitelist.clone();
    for server in &config.mcp_gateway.servers {
        server_whitelist
            .entry(server.name.clone())
            .or_insert_with(|| server.clone());
    }

    McpGatewayConfig {
        enabled: config.mcp_gateway.enabled,
        listen: config.mcp_gateway.listen.clone(),
        server_whitelist,
        global_rate_limit: config.mcp_gateway.global_rate_limit,
        metadata: config.mcp_gateway.metadata.clone(),
    }
}

pub fn open_audit_store(config: &AppConfig) -> Result<AuditStore> {
    if !config.audit.enabled {
        return AuditStore::open_in_memory().context("failed to initialize in-memory audit store");
    }

    let db_path = expand_tilde(&config.audit.db_path)?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create audit directory {}", parent.display()))?;
    }

    AuditStore::open(&db_path)
        .with_context(|| format!("failed to open audit database at {}", db_path.display()))
}

pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path == "~" {
        return home_dir();
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        return Ok(home_dir()?.join(stripped));
    }

    Ok(PathBuf::from(path))
}

pub fn effective_proxy_listen(config: &AppConfig, cli_override: Option<&str>) -> String {
    cli_override
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| config.proxy.listen.clone())
}

pub fn effective_mcp_listen(config: &AppConfig, cli_override: Option<&str>) -> String {
    cli_override
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| config.mcp_gateway.listen.clone())
}

pub fn path_status(path: &Path) -> &'static str {
    if path.exists() {
        "present"
    } else {
        "missing"
    }
}

fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))
}

fn default_proxy_listen() -> String {
    DEFAULT_PROXY_LISTEN.to_string()
}

fn default_mcp_listen() -> String {
    DEFAULT_MCP_LISTEN.to_string()
}

fn default_proxy_max_body_size() -> usize {
    DEFAULT_PROXY_MAX_BODY_SIZE
}

fn default_local_llm_provider() -> String {
    "ollama".to_string()
}

fn default_local_llm_endpoint() -> String {
    DEFAULT_LOCAL_LLM_ENDPOINT.to_string()
}

fn default_model_mapping() -> HashMap<String, String> {
    HashMap::from([("default".to_string(), DEFAULT_LOCAL_LLM_MODEL.to_string())])
}

fn default_audit_db_path() -> String {
    DEFAULT_AUDIT_DB_PATH.to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_expands_to_home_directory() {
        let expanded = expand_tilde("~/rules.yaml").expect("tilde expansion should succeed");
        assert!(expanded.ends_with("rules.yaml"));
        assert!(expanded.is_absolute());
    }

    #[test]
    fn app_config_defaults_match_runtime_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.proxy.listen, DEFAULT_PROXY_LISTEN);
        assert_eq!(config.proxy.max_body_size, DEFAULT_PROXY_MAX_BODY_SIZE);
        assert_eq!(config.local_llm.endpoint, DEFAULT_LOCAL_LLM_ENDPOINT);
        assert_eq!(config.audit.db_path, DEFAULT_AUDIT_DB_PATH);
        assert_eq!(
            config.local_llm.resolve_model(None),
            DEFAULT_LOCAL_LLM_MODEL
        );
    }

    #[test]
    fn legacy_mcp_server_list_builds_whitelist() {
        let config: AppConfig = serde_yaml::from_str(
            r#"
mcp_gateway:
  enabled: true
  servers:
    - name: filesystem
      endpoint: http://localhost:3001
"#,
        )
        .expect("config should parse");

        let gateway = build_mcp_gateway_config(&config);
        assert_eq!(gateway.server_whitelist.len(), 1);
        assert!(gateway.server_whitelist.contains_key("filesystem"));
    }
}
