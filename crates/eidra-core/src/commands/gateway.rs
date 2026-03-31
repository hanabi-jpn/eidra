use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use eidra_mcp::gateway::McpGateway;

use crate::runtime::{
    build_mcp_gateway_config, build_scanner, effective_mcp_listen, load_app_config, runtime_paths,
};

pub async fn run(listen_override: Option<&str>) -> Result<()> {
    let paths = runtime_paths()?;
    let app_config = load_app_config(&paths)?;
    let listen = effective_mcp_listen(&app_config, listen_override);

    let scanner = Arc::new(build_scanner(&app_config, true)?);
    let mut gateway_config = build_mcp_gateway_config(&app_config);
    gateway_config.enabled = true;
    gateway_config.listen = listen.clone();

    let addr: SocketAddr = listen
        .parse()
        .with_context(|| format!("invalid MCP gateway listen address: {listen}"))?;

    tracing::info!(
        "Starting Eidra MCP gateway on {} ({} configured server(s))",
        addr,
        gateway_config.server_whitelist.len()
    );

    McpGateway::new(gateway_config, scanner)
        .run(addr)
        .await
        .map_err(|err| anyhow::anyhow!(err.to_string()))
}
