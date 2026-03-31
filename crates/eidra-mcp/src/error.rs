use thiserror::Error;

/// Errors that can occur in the MCP gateway.
#[derive(Debug, Error)]
pub enum McpError {
    /// The requested MCP server is not in the whitelist.
    #[error("MCP server '{server}' is not in the whitelist")]
    ServerNotWhitelisted { server: String },

    /// The requested tool is not allowed on this MCP server.
    #[error("Tool '{tool}' is not allowed on MCP server '{server}'")]
    ToolNotAllowed { server: String, tool: String },

    /// The requested tool is explicitly blocked on this MCP server.
    #[error("Tool '{tool}' is blocked on MCP server '{server}'")]
    ToolBlocked { server: String, tool: String },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded for MCP server '{server}': {limit} requests/min")]
    RateLimitExceeded { server: String, limit: u32 },

    /// The MCP server is disabled in configuration.
    #[error("MCP server '{server}' is disabled")]
    ServerDisabled { server: String },

    /// The MCP gateway itself is disabled.
    #[error("MCP gateway is disabled")]
    GatewayDisabled,

    /// Configuration error.
    #[error("MCP configuration error: {0}")]
    ConfigError(String),

    /// Scan found sensitive data in MCP response.
    #[error("Sensitive data found in MCP response from '{server}': {details}")]
    SensitiveDataInResponse { server: String, details: String },

    /// Semantic policy violation — tool arguments matched a blocked pattern.
    #[error(
        "semantic policy violation: tool '{tool}' blocked — {rule} (pattern: {matched_pattern})"
    )]
    SemanticPolicyViolation {
        tool: String,
        rule: String,
        matched_pattern: String,
    },

    /// Generic I/O or network error.
    #[error("MCP I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Transport/upstream forwarding error.
    #[error("MCP transport error: {0}")]
    Transport(String),

    /// Serialization/deserialization error.
    #[error("MCP serialization error: {0}")]
    Serialization(String),

    /// Custom error for extensibility.
    #[error("MCP error: {0}")]
    Custom(String),
}
