use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use eidra_scan::scanner::Scanner;

use crate::config::McpGatewayConfig;
use crate::error::McpError;
use crate::handler::{self, McpRequest, McpResponse};

/// Per-server rate limiting state.
struct RateLimitState {
    /// Number of requests in the current window.
    count: u32,
    /// Start of the current window.
    window_start: Instant,
}

/// MCP Gateway server that proxies and validates JSON-RPC requests.
pub struct McpGateway {
    config: McpGatewayConfig,
    scanner: Arc<Scanner>,
    /// Per-server rate limit counters. Key is server name.
    rate_limits: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

impl McpGateway {
    /// Create a new MCP Gateway with the given configuration and scanner.
    pub fn new(config: McpGatewayConfig, scanner: Arc<Scanner>) -> Self {
        Self {
            config,
            scanner,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Run the MCP gateway, listening on the given address.
    ///
    /// This is the main server loop. It accepts HTTP connections and
    /// processes JSON-RPC requests according to the gateway configuration.
    pub async fn run(&self, listen_addr: SocketAddr) -> Result<(), McpError> {
        if !self.config.enabled {
            return Err(McpError::GatewayDisabled);
        }

        let listener = TcpListener::bind(listen_addr).await?;
        info!("MCP Gateway listening on {}", listen_addr);

        let config = Arc::new(self.config.clone());
        let scanner = self.scanner.clone();
        let rate_limits = self.rate_limits.clone();

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let config = config.clone();
            let scanner = scanner.clone();
            let rate_limits = rate_limits.clone();

            tokio::spawn(async move {
                let svc = service_fn(move |req| {
                    let config = config.clone();
                    let scanner = scanner.clone();
                    let rate_limits = rate_limits.clone();
                    async move { handle_request(req, &config, &scanner, &rate_limits).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                    error!("Error serving connection from {}: {}", peer_addr, err);
                }
            });
        }
    }
}

/// Check per-server rate limit. Returns Ok(()) if within limit.
fn check_rate_limit(
    rate_limits: &Mutex<HashMap<String, RateLimitState>>,
    server_name: &str,
    limit: u32,
) -> Result<(), McpError> {
    if limit == 0 {
        return Ok(());
    }

    let mut limits = rate_limits
        .lock()
        .map_err(|e| McpError::Custom(format!("Rate limit lock poisoned: {}", e)))?;

    let now = Instant::now();
    let state = limits
        .entry(server_name.to_string())
        .or_insert(RateLimitState {
            count: 0,
            window_start: now,
        });

    // Reset window every 60 seconds
    if now.duration_since(state.window_start).as_secs() >= 60 {
        state.count = 0;
        state.window_start = now;
    }

    if state.count >= limit {
        return Err(McpError::RateLimitExceeded {
            server: server_name.to_string(),
            limit,
        });
    }

    state.count += 1;
    Ok(())
}

/// Handle a single HTTP request to the MCP gateway.
async fn handle_request(
    req: Request<Incoming>,
    config: &McpGatewayConfig,
    scanner: &Scanner,
    rate_limits: &Mutex<HashMap<String, RateLimitState>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Only accept POST for JSON-RPC
    if req.method() != hyper::Method::POST {
        let resp = McpResponse::error(None, -32600, "Only POST method accepted".to_string());
        return Ok(json_response(StatusCode::METHOD_NOT_ALLOWED, &resp));
    }

    // Read body
    let body_bytes = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("Failed to read request body: {}", e);
            let resp = McpResponse::error(None, -32700, "Failed to read request body".to_string());
            return Ok(json_response(StatusCode::BAD_REQUEST, &resp));
        }
    };

    let body_str = String::from_utf8_lossy(&body_bytes);

    // Parse JSON-RPC request
    let mcp_req: McpRequest = match serde_json::from_str(&body_str) {
        Ok(r) => r,
        Err(e) => {
            warn!("Invalid JSON-RPC request: {}", e);
            let resp = McpResponse::error(None, -32700, format!("Parse error: {}", e));
            return Ok(json_response(StatusCode::BAD_REQUEST, &resp));
        }
    };

    // Validate against config (whitelist, ACL)
    if let Err(err) = handler::validate_request(config, &mcp_req) {
        warn!("MCP request blocked: {}", err);
        let resp = McpResponse::error(mcp_req.id.clone(), -32001, err.to_string());
        return Ok(json_response(StatusCode::FORBIDDEN, &resp));
    }

    // Semantic RBAC: validate tool arguments against semantic rules
    if mcp_req.method == "tools/call" {
        if let Some(ref params) = mcp_req.params {
            if let (Some(server_name), Some(tool_name)) = (
                params.get("server_name").and_then(|v| v.as_str()),
                params.get("name").and_then(|v| v.as_str()),
            ) {
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                if let Err(err) =
                    handler::validate_tool_arguments(config, server_name, tool_name, &arguments)
                {
                    warn!("MCP request blocked by semantic policy: {}", err);
                    let resp = McpResponse::error(mcp_req.id.clone(), -32001, err.to_string());
                    return Ok(json_response(StatusCode::FORBIDDEN, &resp));
                }
            }
        }
    }

    // Rate limiting for tool calls
    if mcp_req.method == "tools/call" {
        if let Some(ref params) = mcp_req.params {
            if let Some(server_name) = params.get("server_name").and_then(|v| v.as_str()) {
                // Determine rate limit for this server
                let limit = config
                    .server_whitelist
                    .get(server_name)
                    .map(|s| {
                        if s.rate_limit > 0 {
                            s.rate_limit
                        } else {
                            config.global_rate_limit
                        }
                    })
                    .unwrap_or(config.global_rate_limit);

                if let Err(err) = check_rate_limit(rate_limits, server_name, limit) {
                    warn!("Rate limit exceeded: {}", err);
                    let resp = McpResponse::error(mcp_req.id.clone(), -32002, err.to_string());
                    return Ok(json_response(StatusCode::TOO_MANY_REQUESTS, &resp));
                }
            }
        }
    }

    // Scan the request body for sensitive data
    let findings = scanner.scan(&body_str);
    if !findings.is_empty() {
        info!(
            "Scan found {} findings in MCP request (method: {})",
            findings.len(),
            mcp_req.method
        );
    }

    // Find the target server from the request params
    let server_name = mcp_req
        .params
        .as_ref()
        .and_then(|p| p.get("server_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let server_entry = config.server_whitelist.get(server_name);

    // If we have a server entry with an endpoint, forward upstream
    if let Some(entry) = server_entry {
        let connector = hyper_util::client::legacy::connect::HttpConnector::new();
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(connector);

        let upstream_req = match hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(&entry.endpoint)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body_str.to_string())))
        {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to build upstream request: {}", e);
                let resp = McpResponse::error(
                    mcp_req.id.clone(),
                    -32003,
                    format!("Failed to build upstream request: {}", e),
                );
                return Ok(json_response(StatusCode::INTERNAL_SERVER_ERROR, &resp));
            }
        };

        match client.request(upstream_req).await {
            Ok(upstream_resp) => {
                let resp_body = match upstream_resp.collect().await {
                    Ok(collected) => collected.to_bytes(),
                    Err(e) => {
                        error!("Failed to read upstream response: {}", e);
                        let resp = McpResponse::error(
                            mcp_req.id.clone(),
                            -32003,
                            format!("Upstream response read error: {}", e),
                        );
                        return Ok(json_response(StatusCode::BAD_GATEWAY, &resp));
                    }
                };

                // Scan response for sensitive data
                let resp_str = String::from_utf8_lossy(&resp_body);
                if entry.scan_responses {
                    let resp_findings = scanner.scan(&resp_str);
                    if !resp_findings.is_empty() {
                        warn!(
                            target: "eidra::mcp",
                            findings = resp_findings.len(),
                            "sensitive data detected in MCP upstream response"
                        );
                    }
                }

                // Pass through the upstream response as-is
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(resp_body))
                    .unwrap_or_else(|_| {
                        Response::new(Full::new(Bytes::from(
                            r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#,
                        )))
                    }))
            }
            Err(e) => {
                error!("Upstream request failed: {}", e);
                let resp = McpResponse::error(
                    mcp_req.id.clone(),
                    -32003,
                    format!("Upstream connection error: {}", e),
                );
                Ok(json_response(StatusCode::BAD_GATEWAY, &resp))
            }
        }
    } else {
        // No server entry found or no server_name — return validated response
        let result = serde_json::json!({
            "status": "validated",
            "method": mcp_req.method,
            "findings_count": findings.len(),
            "message": "Request passed gateway validation. No upstream server configured."
        });
        let resp = McpResponse::success(mcp_req.id, result);
        Ok(json_response(StatusCode::OK, &resp))
    }
}

/// Build an HTTP response with JSON body.
fn json_response(status: StatusCode, body: &McpResponse) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(body).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#.to_string()
    });

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap_or_else(|_| {
            Response::new(Full::new(Bytes::from(
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#,
            )))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::McpServerEntry;

    fn test_gateway() -> McpGateway {
        let mut whitelist = HashMap::new();
        whitelist.insert(
            "test-server".to_string(),
            McpServerEntry {
                name: "test-server".to_string(),
                endpoint: "http://localhost:9000".to_string(),
                enabled: true,
                allowed_tools: vec![],
                blocked_tools: vec![],
                rate_limit: 10,
                scan_responses: true,
                tool_rules: vec![],
                metadata: HashMap::new(),
            },
        );

        let config = McpGatewayConfig {
            enabled: true,
            listen: "127.0.0.1:0".to_string(),
            server_whitelist: whitelist,
            global_rate_limit: 60,
            metadata: HashMap::new(),
        };

        McpGateway::new(config, Arc::new(Scanner::with_defaults()))
    }

    #[test]
    fn test_rate_limit_allows_within_limit() {
        let gw = test_gateway();
        let result = check_rate_limit(&gw.rate_limits, "test-server", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limit_blocks_over_limit() {
        let gw = test_gateway();
        for _ in 0..10 {
            check_rate_limit(&gw.rate_limits, "test-server", 10).unwrap();
        }
        let result = check_rate_limit(&gw.rate_limits, "test-server", 10);
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::RateLimitExceeded { server, limit } => {
                assert_eq!(server, "test-server");
                assert_eq!(limit, 10);
            }
            other => panic!("Expected RateLimitExceeded, got: {:?}", other),
        }
    }

    #[test]
    fn test_rate_limit_zero_means_unlimited() {
        let gw = test_gateway();
        for _ in 0..1000 {
            assert!(check_rate_limit(&gw.rate_limits, "test-server", 0).is_ok());
        }
    }

    #[test]
    fn test_gateway_new() {
        let gw = test_gateway();
        assert!(gw.config.enabled);
        assert_eq!(gw.config.server_whitelist.len(), 1);
    }
}
