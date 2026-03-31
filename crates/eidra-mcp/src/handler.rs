use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::McpGatewayConfig;
use crate::error::McpError;

/// A JSON-RPC request conforming to the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// JSON-RPC version, must be "2.0".
    pub jsonrpc: String,

    /// The method being called (e.g., "tools/call", "tools/list").
    pub method: String,

    /// Optional parameters for the method.
    #[serde(default)]
    pub params: Option<serde_json::Value>,

    /// Optional request ID for correlation.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRpcError {
    /// Error code.
    pub code: i64,

    /// Human-readable error message.
    pub message: String,

    /// Optional additional data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// A JSON-RPC response conforming to the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// JSON-RPC version, always "2.0".
    pub jsonrpc: String,

    /// The result, present on success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// The error, present on failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<McpRpcError>,

    /// Request ID for correlation.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

impl McpResponse {
    /// Create a success response.
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<serde_json::Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(McpRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

/// Extract server_name and tool_name from a "tools/call" request's params.
///
/// Expects params like:
/// ```json
/// { "server_name": "filesystem", "name": "read_file", "arguments": { ... } }
/// ```
fn extract_tool_call_info(params: &serde_json::Value) -> Option<(String, String)> {
    let server_name = params.get("server_name")?.as_str()?.to_string();
    let tool_name = params.get("name")?.as_str()?.to_string();
    Some((server_name, tool_name))
}

/// Validate an incoming MCP request against the gateway configuration.
///
/// Checks:
/// 1. Gateway is enabled
/// 2. For "tools/call" requests: server is whitelisted, enabled, and tool is allowed
///
/// Returns `Ok(())` if the request is allowed, or an appropriate `McpError`.
pub fn validate_request(config: &McpGatewayConfig, req: &McpRequest) -> Result<(), McpError> {
    // Check if gateway is enabled
    if !config.enabled {
        return Err(McpError::GatewayDisabled);
    }

    // Only apply tool-level ACL for "tools/call" method
    if req.method == "tools/call" {
        if let Some(ref params) = req.params {
            if let Some((server_name, tool_name)) = extract_tool_call_info(params) {
                // Check server whitelist
                let server_entry = config.server_whitelist.get(&server_name).ok_or_else(|| {
                    McpError::ServerNotWhitelisted {
                        server: server_name.clone(),
                    }
                })?;

                // Check server is enabled
                if !server_entry.enabled {
                    return Err(McpError::ServerDisabled {
                        server: server_name,
                    });
                }

                // Check blocked tools (takes precedence)
                if server_entry.blocked_tools.contains(&tool_name) {
                    return Err(McpError::ToolBlocked {
                        server: server_name,
                        tool: tool_name,
                    });
                }

                // Check allowed tools (empty means all allowed)
                if !server_entry.allowed_tools.is_empty()
                    && !server_entry.allowed_tools.contains(&tool_name)
                {
                    return Err(McpError::ToolNotAllowed {
                        server: server_name,
                        tool: tool_name,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Validate tool call arguments against semantic rules.
///
/// Inspects the serialized arguments of a tool call and checks them against
/// the `tool_rules` configured for the given server. This enables blocking
/// destructive SQL, sensitive file paths, dangerous shell commands, and
/// secrets embedded in any tool arguments.
///
/// Returns `Ok(())` if allowed, or `Err(McpError::SemanticPolicyViolation)` if blocked.
pub fn validate_tool_arguments(
    config: &McpGatewayConfig,
    server_name: &str,
    tool_name: &str,
    arguments: &serde_json::Value,
) -> Result<(), McpError> {
    let server = config.server_whitelist.get(server_name);
    let server = match server {
        Some(s) => s,
        None => return Ok(()), // No server config = no rules
    };

    // Find matching tool rules
    for rule in &server.tool_rules {
        if rule.tool != tool_name && rule.tool != "*" {
            continue;
        }

        let args_str = arguments.to_string();

        // Check block_patterns (regex match on serialized arguments)
        for pattern in &rule.block_patterns {
            let re = Regex::new(pattern).map_err(|e| {
                McpError::ConfigError(format!("invalid regex '{}': {}", pattern, e))
            })?;
            if re.is_match(&args_str) {
                return Err(McpError::SemanticPolicyViolation {
                    tool: tool_name.to_string(),
                    rule: rule.description.clone(),
                    matched_pattern: pattern.clone(),
                });
            }
        }

        // Check blocked_paths (for file/path arguments)
        if !rule.blocked_paths.is_empty() {
            check_path_access(
                tool_name,
                arguments,
                &rule.blocked_paths,
                &rule.allowed_paths,
            )?;
        }
    }

    Ok(())
}

/// Check whether any path-like strings in the arguments match blocked path patterns.
///
/// If a path matches a blocked pattern, it is still allowed if it also matches
/// an `allowed_paths` entry (allow takes precedence over block for paths).
fn check_path_access(
    tool_name: &str,
    arguments: &serde_json::Value,
    blocked_paths: &[String],
    allowed_paths: &[String],
) -> Result<(), McpError> {
    let paths = extract_paths(arguments);

    for path in &paths {
        for blocked in blocked_paths {
            if glob_match(blocked, path) {
                // Check if explicitly allowed
                let is_allowed = allowed_paths.iter().any(|a| glob_match(a, path));
                if !is_allowed {
                    return Err(McpError::SemanticPolicyViolation {
                        tool: tool_name.to_string(),
                        rule: format!("path '{}' matches blocked pattern '{}'", path, blocked),
                        matched_pattern: blocked.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Extract path-like strings from JSON arguments.
///
/// Looks for string values that start with `/`, `~/`, `./`, or contain `..`.
/// Also extracts values from keys commonly used for file paths
/// (`path`, `file`, `filename`, `directory`).
fn extract_paths(value: &serde_json::Value) -> Vec<String> {
    let mut paths = Vec::new();
    match value {
        serde_json::Value::String(s) => {
            if s.starts_with('/') || s.starts_with("~/") || s.starts_with("./") || s.contains("..")
            {
                paths.push(s.clone());
            }
        }
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                if key == "path" || key == "file" || key == "filename" || key == "directory" {
                    if let Some(s) = val.as_str() {
                        paths.push(s.to_string());
                    }
                }
                paths.extend(extract_paths(val));
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                paths.extend(extract_paths(item));
            }
        }
        _ => {}
    }
    paths
}

/// Simple glob matching for path patterns.
///
/// Supports:
/// - `**` — matches any path (prefix match up to `**`)
/// - Trailing `*` — prefix match
/// - `~` expansion to `$HOME`
/// - Exact match otherwise
fn glob_match(pattern: &str, path: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let pattern = pattern.replace('~', &home);
    let path_expanded = path.replace('~', &home);

    if pattern.contains("**") {
        let prefix = pattern.split("**").next().unwrap_or("");
        return path_expanded.starts_with(prefix);
    }
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return path_expanded.starts_with(prefix);
    }
    path_expanded == pattern
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{McpGatewayConfig, McpServerEntry, McpToolRule};
    use std::collections::HashMap;

    /// Helper to build a test config with one server.
    fn test_config(
        enabled: bool,
        server_name: &str,
        allowed_tools: Vec<String>,
        blocked_tools: Vec<String>,
    ) -> McpGatewayConfig {
        let mut whitelist = HashMap::new();
        whitelist.insert(
            server_name.to_string(),
            McpServerEntry {
                name: server_name.to_string(),
                endpoint: "http://localhost:9000".to_string(),
                enabled: true,
                allowed_tools,
                blocked_tools,
                rate_limit: 0,
                scan_responses: true,
                tool_rules: vec![],
                metadata: HashMap::new(),
            },
        );
        McpGatewayConfig {
            enabled,
            listen: "127.0.0.1:8081".to_string(),
            server_whitelist: whitelist,
            global_rate_limit: 0,
            metadata: HashMap::new(),
        }
    }

    fn tools_call_request(server_name: &str, tool_name: &str) -> McpRequest {
        McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "server_name": server_name,
                "name": tool_name,
                "arguments": {}
            })),
            id: Some(serde_json::json!(1)),
        }
    }

    #[test]
    fn test_validate_allowed_tool() {
        let config = test_config(true, "filesystem", vec![], vec![]);
        let req = tools_call_request("filesystem", "read_file");
        assert!(validate_request(&config, &req).is_ok());
    }

    #[test]
    fn test_validate_blocked_tool() {
        let config = test_config(true, "filesystem", vec![], vec!["delete_file".to_string()]);
        let req = tools_call_request("filesystem", "delete_file");
        let err = validate_request(&config, &req).unwrap_err();
        match err {
            McpError::ToolBlocked { server, tool } => {
                assert_eq!(server, "filesystem");
                assert_eq!(tool, "delete_file");
            }
            other => panic!("Expected ToolBlocked, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_unknown_server() {
        let config = test_config(true, "filesystem", vec![], vec![]);
        let req = tools_call_request("unknown_server", "read_file");
        let err = validate_request(&config, &req).unwrap_err();
        match err {
            McpError::ServerNotWhitelisted { server } => {
                assert_eq!(server, "unknown_server");
            }
            other => panic!("Expected ServerNotWhitelisted, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_tool_not_in_allowed_list() {
        let config = test_config(true, "filesystem", vec!["read_file".to_string()], vec![]);
        let req = tools_call_request("filesystem", "write_file");
        let err = validate_request(&config, &req).unwrap_err();
        match err {
            McpError::ToolNotAllowed { server, tool } => {
                assert_eq!(server, "filesystem");
                assert_eq!(tool, "write_file");
            }
            other => panic!("Expected ToolNotAllowed, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_gateway_disabled() {
        let config = test_config(false, "filesystem", vec![], vec![]);
        let req = tools_call_request("filesystem", "read_file");
        let err = validate_request(&config, &req).unwrap_err();
        assert!(matches!(err, McpError::GatewayDisabled));
    }

    #[test]
    fn test_validate_non_tool_call_passes() {
        let config = test_config(true, "filesystem", vec![], vec![]);
        let req = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        };
        assert!(validate_request(&config, &req).is_ok());
    }

    #[test]
    fn test_blocked_takes_precedence_over_allowed() {
        let config = test_config(
            true,
            "filesystem",
            vec!["delete_file".to_string()],
            vec!["delete_file".to_string()],
        );
        let req = tools_call_request("filesystem", "delete_file");
        let err = validate_request(&config, &req).unwrap_err();
        assert!(matches!(err, McpError::ToolBlocked { .. }));
    }

    #[test]
    fn test_server_disabled() {
        let mut config = test_config(true, "filesystem", vec![], vec![]);
        config
            .server_whitelist
            .get_mut("filesystem")
            .unwrap()
            .enabled = false;
        let req = tools_call_request("filesystem", "read_file");
        let err = validate_request(&config, &req).unwrap_err();
        assert!(matches!(err, McpError::ServerDisabled { .. }));
    }

    // --- Semantic RBAC tests ---

    /// Helper to build a config with semantic tool rules on a given server.
    fn make_config_with_rules(server_name: &str, rules: Vec<McpToolRule>) -> McpGatewayConfig {
        let mut whitelist = HashMap::new();
        whitelist.insert(
            server_name.to_string(),
            McpServerEntry {
                name: server_name.to_string(),
                endpoint: "http://localhost:9000".to_string(),
                enabled: true,
                allowed_tools: vec![],
                blocked_tools: vec![],
                rate_limit: 0,
                scan_responses: true,
                tool_rules: rules,
                metadata: HashMap::new(),
            },
        );
        McpGatewayConfig {
            enabled: true,
            listen: "127.0.0.1:8081".to_string(),
            server_whitelist: whitelist,
            global_rate_limit: 0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_semantic_block_destructive_sql() {
        let config = make_config_with_rules(
            "db_server",
            vec![McpToolRule {
                tool: "execute_sql".to_string(),
                block_patterns: vec![r"(?i)\b(DROP|DELETE|TRUNCATE|ALTER)\b".to_string()],
                blocked_paths: vec![],
                allowed_paths: vec![],
                description: "Block destructive SQL".to_string(),
            }],
        );

        let args = serde_json::json!({"query": "DROP TABLE users"});
        let result = validate_tool_arguments(&config, "db_server", "execute_sql", &args);
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::SemanticPolicyViolation { tool, rule, .. } => {
                assert_eq!(tool, "execute_sql");
                assert_eq!(rule, "Block destructive SQL");
            }
            other => panic!("Expected SemanticPolicyViolation, got: {:?}", other),
        }

        let args = serde_json::json!({"query": "SELECT * FROM users"});
        let result = validate_tool_arguments(&config, "db_server", "execute_sql", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_block_sensitive_path() {
        let config = make_config_with_rules(
            "fs_server",
            vec![McpToolRule {
                tool: "read_file".to_string(),
                block_patterns: vec![],
                blocked_paths: vec![
                    "~/.ssh/**".to_string(),
                    "~/.aws/**".to_string(),
                    "/etc/shadow".to_string(),
                ],
                allowed_paths: vec![],
                description: "Block sensitive paths".to_string(),
            }],
        );

        let args = serde_json::json!({"path": "/etc/shadow"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_err());

        let args = serde_json::json!({"path": "/home/user/code/main.rs"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_block_destructive_command() {
        let config = make_config_with_rules(
            "shell",
            vec![McpToolRule {
                tool: "run_command".to_string(),
                block_patterns: vec![
                    r"rm\s+(-rf?|--recursive)".to_string(),
                    r"curl.*\|\s*(sh|bash)".to_string(),
                    r"chmod\s+777".to_string(),
                ],
                blocked_paths: vec![],
                allowed_paths: vec![],
                description: "Block destructive commands".to_string(),
            }],
        );

        let args = serde_json::json!({"command": "rm -rf /"});
        let result = validate_tool_arguments(&config, "shell", "run_command", &args);
        assert!(result.is_err());

        let args = serde_json::json!({"command": "ls -la"});
        let result = validate_tool_arguments(&config, "shell", "run_command", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_wildcard_rule_applies_to_all_tools() {
        let config = make_config_with_rules(
            "any_server",
            vec![McpToolRule {
                tool: "*".to_string(),
                block_patterns: vec![
                    r#"(?i)(password|secret|token|api.?key)\s*[:=]\s*['"]?[A-Za-z0-9]{8,}"#
                        .to_string(),
                ],
                blocked_paths: vec![],
                allowed_paths: vec![],
                description: "Block secrets in any tool arguments".to_string(),
            }],
        );

        let args = serde_json::json!({"data": "my api_key=ABCDEF1234567890"});
        let result = validate_tool_arguments(&config, "any_server", "some_tool", &args);
        assert!(result.is_err());

        let args = serde_json::json!({"data": "hello world"});
        let result = validate_tool_arguments(&config, "any_server", "some_tool", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_allowed_path_overrides_blocked() {
        let config = make_config_with_rules(
            "fs_server",
            vec![McpToolRule {
                tool: "read_file".to_string(),
                block_patterns: vec![],
                blocked_paths: vec!["/etc/**".to_string()],
                allowed_paths: vec!["/etc/hostname".to_string()],
                description: "Block /etc except hostname".to_string(),
            }],
        );

        // /etc/shadow is blocked
        let args = serde_json::json!({"path": "/etc/shadow"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_err());

        // /etc/hostname is explicitly allowed
        let args = serde_json::json!({"path": "/etc/hostname"});
        let result = validate_tool_arguments(&config, "fs_server", "read_file", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_no_rules_passes() {
        let config = make_config_with_rules("server", vec![]);
        let args = serde_json::json!({"query": "DROP TABLE users"});
        let result = validate_tool_arguments(&config, "server", "execute_sql", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_unknown_server_passes() {
        let config = make_config_with_rules("known_server", vec![]);
        let args = serde_json::json!({"query": "DROP TABLE users"});
        let result = validate_tool_arguments(&config, "unknown_server", "execute_sql", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_paths_from_nested_json() {
        let val = serde_json::json!({
            "options": {
                "path": "/etc/passwd",
                "other": "not a path"
            }
        });
        let paths = extract_paths(&val);
        assert!(paths.contains(&"/etc/passwd".to_string()));
        assert!(!paths.contains(&"not a path".to_string()));
    }

    #[test]
    fn test_extract_paths_from_array() {
        let val = serde_json::json!({
            "files": [
                {"path": "/tmp/a.txt"},
                {"file": "/var/log/b.log"}
            ]
        });
        let paths = extract_paths(&val);
        assert!(paths.contains(&"/tmp/a.txt".to_string()));
        assert!(paths.contains(&"/var/log/b.log".to_string()));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("/etc/shadow", "/etc/shadow"));
        assert!(!glob_match("/etc/shadow", "/etc/passwd"));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("/etc/*", "/etc/shadow"));
        assert!(!glob_match("/etc/*", "/var/log/syslog"));
    }

    #[test]
    fn test_glob_match_double_star() {
        assert!(glob_match("/etc/**", "/etc/shadow"));
        assert!(glob_match("/etc/**", "/etc/ssl/certs/ca.pem"));
        assert!(!glob_match("/etc/**", "/var/log/syslog"));
    }
}
