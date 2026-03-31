use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use eidra_audit::store::AuditStore;
use eidra_policy::engine::PolicyEngine;
use eidra_router::ollama::OllamaRouter;
use eidra_scan::scanner::Scanner;

use crate::error::ProxyError;
use crate::handler::{handle_request, HandlerRuntime};
use crate::tls::CaAuthority;
use crate::EventSender;

pub struct ProxyConfig {
    pub listen_addr: SocketAddr,
    pub max_body_size: usize,
    pub metadata: HashMap<String, String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: ([127, 0, 0, 1], 8080).into(),
            max_body_size: 10 * 1024 * 1024,
            metadata: HashMap::new(),
        }
    }
}

pub async fn run_proxy(
    config: ProxyConfig,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    ca: Option<Arc<CaAuthority>>,
    local_router: Option<Arc<OllamaRouter>>,
) -> Result<(), ProxyError> {
    let listener = TcpListener::bind(config.listen_addr).await?;

    if ca.is_some() {
        tracing::info!(
            target: "eidra::proxy",
            addr = %config.listen_addr,
            "Proxy listening (HTTPS MITM enabled for AI providers)"
        );
    } else {
        tracing::info!(
            target: "eidra::proxy",
            addr = %config.listen_addr,
            "Proxy listening (HTTP-only mode — no CA loaded)"
        );
    }

    loop {
        let (stream, addr) = listener.accept().await?;
        let scanner = scanner.clone();
        let policy = policy.clone();
        let audit = audit.clone();
        let event_tx = event_tx.clone();
        let ca = ca.clone();
        let local_router = local_router.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let runtime = HandlerRuntime::new(
                scanner.clone(),
                policy.clone(),
                audit.clone(),
                event_tx.clone(),
                local_router.clone(),
                config.max_body_size,
            );
            let ca = ca.clone();

            let service = service_fn(move |req| {
                let runtime = runtime.clone();
                let ca = ca.clone();
                async move { handle_request(req, runtime, ca).await }
            });

            if let Err(e) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                tracing::debug!(
                    target: "eidra::proxy",
                    addr = %addr,
                    error = %e,
                    "connection error"
                );
            }
        });
    }
}
