use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the MCP (Model Context Protocol) gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpGatewayConfig {
    /// Whether the MCP gateway is enabled.
    #[serde(default)]
    pub enabled: bool,

    /// Address to listen on (e.g., "127.0.0.1:8081").
    #[serde(default = "default_listen")]
    pub listen: String,

    /// Whitelist of allowed MCP servers. Key is the server name/id.
    #[serde(default)]
    pub server_whitelist: HashMap<String, McpServerEntry>,

    /// Global rate limit (requests per minute). 0 means unlimited.
    #[serde(default)]
    pub global_rate_limit: u32,

    /// Extensible metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// A semantic argument-level access control rule for an MCP tool.
///
/// Instead of just allowing/blocking tools by name, this inspects the
/// arguments of tool calls and blocks based on semantic patterns (e.g.,
/// destructive SQL, sensitive file paths, dangerous shell commands).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRule {
    /// Tool name this rule applies to (e.g., "execute_sql", "read_file", "run_command").
    /// Use "*" to match all tools on this server.
    pub tool: String,

    /// Regex patterns to block in serialized tool arguments.
    #[serde(default)]
    pub block_patterns: Vec<String>,

    /// Allowed path patterns (for file access tools). Paths matching these
    /// are permitted even if they match a `blocked_paths` pattern.
    #[serde(default)]
    pub allowed_paths: Vec<String>,

    /// Blocked path patterns (for file access tools).
    #[serde(default)]
    pub blocked_paths: Vec<String>,

    /// Human-readable description of this rule.
    #[serde(default)]
    pub description: String,
}

/// An entry in the MCP server whitelist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntry {
    /// Human-readable name of the MCP server.
    pub name: String,

    /// The endpoint URL of the MCP server.
    pub endpoint: String,

    /// Whether this server is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Allowed tools for this server. Empty means all tools allowed.
    #[serde(default)]
    pub allowed_tools: Vec<String>,

    /// Blocked tools for this server. Takes precedence over allowed_tools.
    #[serde(default)]
    pub blocked_tools: Vec<String>,

    /// Per-server rate limit (requests per minute). 0 means use global.
    #[serde(default)]
    pub rate_limit: u32,

    /// Whether to scan responses from this server.
    #[serde(default = "default_true")]
    pub scan_responses: bool,

    /// Semantic argument-level access control rules for tools on this server.
    #[serde(default)]
    pub tool_rules: Vec<McpToolRule>,

    /// Extensible metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for McpGatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen: default_listen(),
            server_whitelist: HashMap::new(),
            global_rate_limit: 0,
            metadata: HashMap::new(),
        }
    }
}

fn default_listen() -> String {
    "127.0.0.1:8081".to_string()
}

fn default_true() -> bool {
    true
}
