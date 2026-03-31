use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use eidra_policy::engine::PolicyEngine;
use eidra_proxy::create_event_channel;
use eidra_proxy::server::{run_proxy, ProxyConfig};
use eidra_proxy::tls::CaAuthority;

use crate::runtime::{
    build_local_router, build_scanner, effective_proxy_listen, load_app_config, load_policy_config,
    open_audit_store, runtime_paths,
};

pub async fn run(listen_override: Option<&str>, dashboard: bool) -> anyhow::Result<()> {
    let paths = runtime_paths()?;
    let app_config = load_app_config(&paths)?;
    let listen = effective_proxy_listen(&app_config, listen_override);

    // Initialize scanner from runtime configuration
    let scanner = Arc::new(build_scanner(&app_config, app_config.scan.enabled)?);
    tracing::info!(
        "Scan engine loaded ({} classifiers)",
        scanner.classifier_count()
    );

    // Initialize policy engine
    let policy_config = load_policy_config(&paths)?;
    let policy = Arc::new(PolicyEngine::new(policy_config));
    tracing::info!("Policy engine loaded");

    // Initialize audit store
    std::fs::create_dir_all(&paths.eidra_dir)?;
    let audit = Arc::new(open_audit_store(&app_config)?);
    tracing::info!(
        "Audit store initialized ({})",
        if app_config.audit.enabled {
            &app_config.audit.db_path
        } else {
            "in-memory"
        }
    );

    // Load CA for HTTPS MITM (optional — degrades to HTTP-only mode if absent)
    let ca = if paths.ca_cert_path.exists() && paths.ca_key_path.exists() {
        match CaAuthority::load(&paths.ca_cert_path, &paths.ca_key_path) {
            Ok(ca) => {
                tracing::info!(
                    "CA loaded from {} — HTTPS MITM enabled for AI providers",
                    paths.eidra_dir.display()
                );
                Some(Arc::new(ca))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load CA from {}: {} — continuing in HTTP-only mode",
                    paths.eidra_dir.display(),
                    e
                );
                None
            }
        }
    } else {
        tracing::warn!(
            "CA files not found at {} — HTTPS MITM disabled. \
             Run `eidra init` to generate CA certificates, then trust the CA in your OS.",
            paths.eidra_dir.display()
        );
        None
    };

    let local_router = build_local_router(&app_config)?.map(Arc::new);
    if let Some(router) = local_router.as_ref() {
        tracing::info!(
            "Local LLM routing enabled via {} at {}",
            router.model(),
            router.endpoint()
        );
    }

    // Create event channel for TUI
    let (event_tx, event_rx) = create_event_channel();

    // Start proxy
    let addr: SocketAddr = listen
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid listen address: {}", e))?;
    let config = ProxyConfig {
        listen_addr: addr,
        max_body_size: app_config.proxy.max_body_size,
        metadata: HashMap::new(),
    };

    // Write PID file for `eidra stop`
    std::fs::write(&paths.pid_path, std::process::id().to_string())?;

    if dashboard {
        // Run proxy in background, TUI in foreground
        tracing::info!("Starting Eidra proxy on {} with dashboard", addr);
        let proxy_handle = tokio::spawn(async move {
            if let Err(e) =
                run_proxy(config, scanner, policy, audit, event_tx, ca, local_router).await
            {
                tracing::error!("Proxy error: {}", e);
            }
        });

        // Run TUI (this blocks until user quits)
        super::dashboard::run_tui(event_rx).await?;

        proxy_handle.abort();
    } else {
        tracing::info!("Starting Eidra proxy on {}", addr);
        run_proxy(config, scanner, policy, audit, event_tx, ca, local_router)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    Ok(())
}
