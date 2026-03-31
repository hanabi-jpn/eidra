use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Method, Request, Response, StatusCode};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

use eidra_audit::event::{ActionTaken, AuditEvent, EventType};
use eidra_audit::store::AuditStore;
use eidra_policy::engine::PolicyEngine;
use eidra_policy::types::{PolicyAction, PolicyContext, RouteTarget};
use eidra_router::masking::mask_findings;
use eidra_router::ollama::OllamaRouter;
use eidra_scan::scanner::Scanner;

use crate::ai_domains::is_ai_provider;
use crate::tls::CaAuthority;
use crate::{EventSender, ProxyEvent};

fn send_event(event_tx: &EventSender, event: ProxyEvent) {
    if event_tx.send(event).is_err() {
        tracing::debug!(target: "eidra::proxy", "event channel: no active receivers");
    }
}

fn build_proxy_event(
    action: &str,
    provider: &str,
    findings_count: u32,
    categories: Vec<String>,
    data_size_bytes: u64,
    latency_ms: u64,
    status_code: u16,
) -> ProxyEvent {
    ProxyEvent {
        timestamp: chrono::Utc::now(),
        action: action.to_string(),
        provider: provider.to_string(),
        findings_count,
        categories,
        data_size_bytes,
        latency_ms,
        status_code,
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

#[derive(Clone)]
pub struct HandlerRuntime {
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    local_router: Option<Arc<OllamaRouter>>,
    max_body_size: usize,
}

impl HandlerRuntime {
    pub fn new(
        scanner: Arc<Scanner>,
        policy: Arc<PolicyEngine>,
        audit: Arc<AuditStore>,
        event_tx: EventSender,
        local_router: Option<Arc<OllamaRouter>>,
        max_body_size: usize,
    ) -> Self {
        Self {
            scanner,
            policy,
            audit,
            event_tx,
            local_router,
            max_body_size,
        }
    }
}

fn full_body(data: impl Into<Bytes>) -> BoxBody {
    Full::new(data.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty_body() -> BoxBody {
    Full::new(Bytes::new())
        .map_err(|never| match never {})
        .boxed()
}

pub async fn handle_request(
    req: Request<Incoming>,
    runtime: HandlerRuntime,
    ca: Option<Arc<CaAuthority>>,
) -> Result<Response<BoxBody>, hyper::Error> {
    if req.method() == Method::CONNECT {
        handle_connect(
            req,
            runtime.scanner,
            runtime.policy,
            runtime.audit,
            runtime.event_tx,
            ca,
            runtime.local_router,
            runtime.max_body_size,
        )
        .await
    } else {
        handle_http(
            req,
            runtime.scanner,
            runtime.policy,
            runtime.audit,
            runtime.event_tx,
            runtime.local_router,
            runtime.max_body_size,
        )
        .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_connect(
    req: Request<Incoming>,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    ca: Option<Arc<CaAuthority>>,
    local_router: Option<Arc<OllamaRouter>>,
    max_body_size: usize,
) -> Result<Response<BoxBody>, hyper::Error> {
    let host = req
        .uri()
        .authority()
        .map(|a| a.to_string())
        .unwrap_or_default();

    let is_ai = is_ai_provider(&host);

    if is_ai {
        tracing::info!(target: "eidra::proxy", host = %host, "AI provider detected (CONNECT tunnel — MITM)");
    } else {
        tracing::debug!(target: "eidra::proxy", host = %host, "CONNECT tunnel (passthrough)");
    }

    // Only MITM AI providers, and only if we have a CA loaded
    let should_mitm = is_ai && ca.is_some();

    if should_mitm {
        let ca = ca.expect("checked is_some above");
        let host_clone = host.clone();

        tokio::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = mitm_tunnel(
                        upgraded,
                        &host_clone,
                        ca,
                        scanner,
                        policy,
                        audit,
                        event_tx,
                        local_router,
                        max_body_size,
                    )
                    .await
                    {
                        tracing::error!(target: "eidra::proxy", error = %e, host = %host_clone, "MITM tunnel error");
                    }
                }
                Err(e) => {
                    tracing::error!(target: "eidra::proxy", error = %e, "upgrade error");
                }
            }
        });
    } else {
        tokio::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, &host).await {
                        tracing::error!(target: "eidra::proxy", error = %e, "tunnel error");
                    }
                }
                Err(e) => {
                    tracing::error!(target: "eidra::proxy", error = %e, "upgrade error");
                }
            }
        });
    }

    Ok(Response::new(empty_body()))
}

/// Transparent byte-level tunnel for non-AI CONNECT requests.
async fn tunnel(
    upgraded: hyper::upgrade::Upgraded,
    host: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut server = TcpStream::connect(host).await?;
    let mut upgraded = hyper_util::rt::TokioIo::new(upgraded);
    tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
    Ok(())
}

/// MITM tunnel for AI provider CONNECT requests.
///
/// Uses hyper's native HTTP/1.1 framing instead of manual byte parsing.
/// This eliminates HTTP smuggling vulnerabilities and correctly handles
/// chunked encoding, content-length, and all HTTP framing edge cases.
///
/// 1. Accept TLS from client using a dynamically generated certificate
/// 2. Use hyper to serve the decrypted connection as an HTTP server
/// 3. For each request: scan, apply policy, forward upstream via hyper client
#[allow(clippy::too_many_arguments)]
async fn mitm_tunnel(
    upgraded: hyper::upgrade::Upgraded,
    host: &str,
    ca: Arc<CaAuthority>,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    local_router: Option<Arc<OllamaRouter>>,
    max_body_size: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let hostname = host.split(':').next().unwrap_or(host);
    let port = host
        .split(':')
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(443);

    // Generate TLS cert for this domain
    let server_config = ca.server_config_for_domain(hostname)?;
    let acceptor = TlsAcceptor::from(server_config);

    // Accept TLS from client
    let upgraded_io = hyper_util::rt::TokioIo::new(upgraded);
    let client_tls = acceptor
        .accept(upgraded_io)
        .await
        .map_err(|e| format!("TLS accept failed for {}: {}", hostname, e))?;

    tracing::debug!(target: "eidra::proxy", host = %hostname, "TLS accepted from client");

    // Wrap the TLS stream in TokioIo for hyper
    let client_io = hyper_util::rt::TokioIo::new(client_tls);

    // Use hyper to serve the connection — this handles HTTP framing natively
    let hostname_owned = hostname.to_string();
    let port_owned = port;

    let service = hyper::service::service_fn(move |req: Request<Incoming>| {
        let scanner = scanner.clone();
        let policy = policy.clone();
        let audit = audit.clone();
        let event_tx = event_tx.clone();
        let hostname = hostname_owned.clone();
        let local_router = local_router.clone();
        let max_body_size = max_body_size;

        async move {
            handle_mitm_request(
                req,
                &hostname,
                port_owned,
                scanner,
                policy,
                audit,
                event_tx,
                local_router,
                max_body_size,
            )
            .await
        }
    });

    // Serve one HTTP/1.1 connection (may handle multiple requests via keep-alive)
    if let Err(e) = hyper::server::conn::http1::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .serve_connection(client_io, service)
        .await
    {
        tracing::debug!(target: "eidra::proxy", error = %e, "MITM connection ended");
    }

    Ok(())
}

/// Handle a single request in MITM mode — scan, apply policy, forward upstream via HTTPS.
#[allow(clippy::too_many_arguments)]
async fn handle_mitm_request(
    req: Request<Incoming>,
    hostname: &str,
    port: u16,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    local_router: Option<Arc<OllamaRouter>>,
    max_body_size: usize,
) -> Result<Response<BoxBody>, hyper::Error> {
    // Collect the full body (hyper handles chunked/content-length natively)
    let (parts, body) = req.into_parts();
    let body_bytes = body.collect().await?.to_bytes();
    let body_size = body_bytes.len() as u64;
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    let request_path = parts.uri.path().to_string();
    let request_started = Instant::now();

    tracing::info!(
        target: "eidra::proxy",
        host = %hostname,
        body_size = body_size,
        "MITM: intercepted HTTPS request"
    );

    if body_bytes.len() > max_body_size {
        tracing::warn!(
            target: "eidra::proxy",
            host = %hostname,
            body_size = body_size,
            max_body_size = max_body_size,
            "MITM: request exceeded configured body size limit"
        );
        return Ok(body_too_large_response(
            &audit,
            &event_tx,
            hostname,
            body_size,
            max_body_size,
            elapsed_ms(request_started),
        ));
    }

    // Scan the request body
    let findings = scanner.scan(&body_str);

    // Log findings
    for finding in &findings {
        tracing::warn!(
            target: "eidra::proxy",
            rule = %finding.rule_name,
            category = %finding.category,
            severity = %finding.severity,
            "MITM Finding detected"
        );
    }

    // If no findings, forward as-is
    if findings.is_empty() {
        tracing::info!(target: "eidra::proxy", host = %hostname, "MITM: clean request");
        let event = AuditEvent::new(
            EventType::AiRequest,
            ActionTaken::Allow,
            hostname,
            0,
            "[]",
            body_size,
        );
        let _ = audit.log_event(&event);

        let response = forward_upstream_tls(parts, body_bytes, hostname, port).await?;
        send_event(
            &event_tx,
            build_proxy_event(
                "allow",
                hostname,
                0,
                vec![],
                body_size,
                elapsed_ms(request_started),
                response.status().as_u16(),
            ),
        );
        return Ok(response);
    }

    // Apply policy
    let ctx = PolicyContext {
        findings: &findings,
        destination: hostname,
        data_size_bytes: body_size,
        metadata: HashMap::new(),
    };
    let decision = policy.evaluate(&ctx);
    let categories: Vec<String> = findings.iter().map(|f| f.category.to_string()).collect();

    let (final_body, audit_action) = match decision.overall_action {
        PolicyAction::Block => {
            tracing::warn!(target: "eidra::proxy", host = %hostname, "MITM: BLOCKED");
            send_event(
                &event_tx,
                build_proxy_event(
                    "block",
                    hostname,
                    findings.len() as u32,
                    categories.clone(),
                    body_size,
                    elapsed_ms(request_started),
                    StatusCode::FORBIDDEN.as_u16(),
                ),
            );
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Block,
                hostname,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);

            let block_body = serde_json::json!({
                "error": "blocked_by_eidra",
                "message": "Request blocked: contains sensitive data that violates security policy",
                "findings_count": findings.len(),
                "categories": categories,
            });
            let mut resp = Response::new(full_body(block_body.to_string()));
            *resp.status_mut() = StatusCode::FORBIDDEN;
            return Ok(resp);
        }
        PolicyAction::Escalate => {
            tracing::warn!(
                target: "eidra::proxy",
                host = %hostname,
                "MITM: escalation required"
            );
            log_decision(
                &audit,
                &event_tx,
                ActionTaken::Escalate,
                "escalate",
                hostname,
                findings.len() as u32,
                categories.clone(),
                body_size,
                elapsed_ms(request_started),
                StatusCode::FORBIDDEN.as_u16(),
            );
            return Ok(escalate_response(findings.len(), categories));
        }
        PolicyAction::Mask => {
            tracing::info!(target: "eidra::proxy", host = %hostname, findings = findings.len(), "MITM: masking");
            let masked = mask_findings(&body_str, &findings);
            let masked_bytes = Bytes::from(masked);
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Mask,
                hostname,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);

            (masked_bytes, "mask")
        }
        PolicyAction::Route(RouteTarget::Local) => {
            tracing::info!(
                target: "eidra::proxy",
                host = %hostname,
                findings = findings.len(),
                "MITM: routing to local LLM"
            );
            return route_to_local_llm(
                local_router,
                hostname,
                &request_path,
                &body_str,
                &audit,
                &event_tx,
                findings.len() as u32,
                categories,
                body_size,
                request_started,
            )
            .await;
        }
        _ => {
            tracing::info!(target: "eidra::proxy", host = %hostname, "MITM: allowing with findings");
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Allow,
                hostname,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);
            (body_bytes, "allow")
        }
    };

    let response = forward_upstream_tls(parts, final_body, hostname, port).await?;
    send_event(
        &event_tx,
        build_proxy_event(
            audit_action,
            hostname,
            findings.len() as u32,
            categories,
            body_size,
            elapsed_ms(request_started),
            response.status().as_u16(),
        ),
    );
    Ok(response)
}

/// Forward a request to the real upstream server via HTTPS (TLS client).
///
/// Uses hyper-rustls for proper HTTP/1.1 framing over TLS, replacing
/// the previous manual byte-level forwarding.
async fn forward_upstream_tls(
    parts: hyper::http::request::Parts,
    body: Bytes,
    hostname: &str,
    port: u16,
) -> Result<Response<BoxBody>, hyper::Error> {
    // Build upstream URI
    let path = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let upstream_uri: hyper::Uri = format!("https://{}:{}{}", hostname, port, path)
        .parse()
        .unwrap_or_else(|_| format!("https://{}{}", hostname, path).parse().unwrap());

    // Create TLS connector
    let client_config = match crate::tls::make_upstream_client_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!(target: "eidra::proxy", error = %e, "failed to create TLS client config");
            let mut resp = Response::new(full_body(format!("TLS error: {}", e)));
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            return Ok(resp);
        }
    };

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(client_config)
        .https_only()
        .enable_http1()
        .build();

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(https_connector);

    // Build the upstream request, preserving original headers
    let mut upstream_req = Request::from_parts(parts, Full::new(body));
    *upstream_req.uri_mut() = upstream_uri;

    match client.request(upstream_req).await {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            let body_bytes = body.collect().await?.to_bytes();
            Ok(Response::from_parts(parts, full_body(body_bytes)))
        }
        Err(e) => {
            tracing::error!(target: "eidra::proxy", error = %e, "upstream TLS request failed");
            let mut resp = Response::new(full_body(format!("Upstream error: {}", e)));
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            Ok(resp)
        }
    }
}

async fn handle_http(
    req: Request<Incoming>,
    scanner: Arc<Scanner>,
    policy: Arc<PolicyEngine>,
    audit: Arc<AuditStore>,
    event_tx: EventSender,
    local_router: Option<Arc<OllamaRouter>>,
    max_body_size: usize,
) -> Result<Response<BoxBody>, hyper::Error> {
    let uri = req.uri().clone();
    let host = uri.host().unwrap_or("unknown").to_string();
    let is_ai = is_ai_provider(&host);

    if is_ai {
        tracing::info!(target: "eidra::proxy", host = %host, uri = %uri, "AI request intercepted");
    }

    // Collect body
    let (parts, body) = req.into_parts();
    let body_bytes = body.collect().await?.to_bytes();
    let body_size = body_bytes.len() as u64;
    let request_path = uri.path().to_string();
    let request_started = Instant::now();

    // If not AI provider, passthrough immediately
    if !is_ai {
        let mut upstream_req = Request::from_parts(parts, Full::new(body_bytes));
        if let Ok(parsed) = uri.to_string().parse() {
            *upstream_req.uri_mut() = parsed;
        }
        return forward_request(upstream_req).await;
    }

    if body_bytes.len() > max_body_size {
        tracing::warn!(
            target: "eidra::proxy",
            host = %host,
            body_size = body_size,
            max_body_size = max_body_size,
            "Request exceeded configured body size limit"
        );
        return Ok(body_too_large_response(
            &audit,
            &event_tx,
            &host,
            body_size,
            max_body_size,
            elapsed_ms(request_started),
        ));
    }

    // Scan
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    let findings = scanner.scan(&body_str);

    if findings.is_empty() {
        tracing::info!(target: "eidra::proxy", host = %host, "No findings — clean request");
        let audit_event = AuditEvent::new(
            EventType::AiRequest,
            ActionTaken::Allow,
            &host,
            0,
            "[]",
            body_size,
        );
        let _ = audit.log_event(&audit_event);

        let mut upstream_req = Request::from_parts(parts, Full::new(body_bytes));
        if let Ok(parsed) = uri.to_string().parse() {
            *upstream_req.uri_mut() = parsed;
        }
        let response = forward_request(upstream_req).await?;
        send_event(
            &event_tx,
            build_proxy_event(
                "allow",
                &host,
                0,
                vec![],
                body_size,
                elapsed_ms(request_started),
                response.status().as_u16(),
            ),
        );
        return Ok(response);
    }

    // Log findings
    for finding in &findings {
        tracing::warn!(
            target: "eidra::proxy",
            rule = %finding.rule_name,
            category = %finding.category,
            severity = %finding.severity,
            "Finding detected"
        );
    }

    // Policy evaluation
    let ctx = PolicyContext {
        findings: &findings,
        destination: &host,
        data_size_bytes: body_size,
        metadata: HashMap::new(),
    };
    let decision = policy.evaluate(&ctx);

    let categories: Vec<String> = findings.iter().map(|f| f.category.to_string()).collect();

    let (final_body, audit_action) = match decision.overall_action {
        PolicyAction::Block => {
            tracing::warn!(target: "eidra::proxy", host = %host, "Request BLOCKED by policy");
            let summary = serde_json::to_string(&categories).unwrap_or_default();
            let event = AuditEvent::new(
                EventType::AiRequest,
                ActionTaken::Block,
                &host,
                findings.len() as u32,
                summary,
                body_size,
            );
            let _ = audit.log_event(&event);
            send_event(
                &event_tx,
                build_proxy_event(
                    "block",
                    &host,
                    findings.len() as u32,
                    categories.clone(),
                    body_size,
                    elapsed_ms(request_started),
                    StatusCode::FORBIDDEN.as_u16(),
                ),
            );

            let block_body = serde_json::json!({
                "error": "blocked_by_eidra",
                "message": "Request blocked: contains sensitive data that violates security policy",
                "findings_count": findings.len(),
                "categories": findings.iter().map(|f| f.category.to_string()).collect::<Vec<_>>(),
            });
            let mut resp = Response::new(full_body(block_body.to_string()));
            *resp.status_mut() = StatusCode::FORBIDDEN;
            return Ok(resp);
        }
        PolicyAction::Escalate => {
            tracing::warn!(
                target: "eidra::proxy",
                host = %host,
                "Request requires escalation"
            );
            log_decision(
                &audit,
                &event_tx,
                ActionTaken::Escalate,
                "escalate",
                &host,
                findings.len() as u32,
                categories.clone(),
                body_size,
                elapsed_ms(request_started),
                StatusCode::FORBIDDEN.as_u16(),
            );
            return Ok(escalate_response(findings.len(), categories));
        }
        PolicyAction::Mask => {
            tracing::info!(target: "eidra::proxy", host = %host, findings = findings.len(), "Masking findings");
            let masked = mask_findings(&body_str, &findings);
            (Bytes::from(masked), ActionTaken::Mask)
        }
        PolicyAction::Route(RouteTarget::Local) => {
            tracing::info!(
                target: "eidra::proxy",
                host = %host,
                findings = findings.len(),
                "Routing request to local LLM"
            );
            return route_to_local_llm(
                local_router,
                &host,
                &request_path,
                &body_str,
                &audit,
                &event_tx,
                findings.len() as u32,
                categories,
                body_size,
                request_started,
            )
            .await;
        }
        _ => {
            tracing::info!(target: "eidra::proxy", host = %host, "Allowing with findings");
            (body_bytes, ActionTaken::Allow)
        }
    };

    // Log audit event
    let summary = serde_json::to_string(&categories).unwrap_or_default();
    let event = AuditEvent::new(
        EventType::AiRequest,
        audit_action,
        &host,
        findings.len() as u32,
        summary,
        body_size,
    );
    let _ = audit.log_event(&event);
    let action_str = match &event.action {
        ActionTaken::Mask => "mask",
        ActionTaken::Block => "block",
        ActionTaken::Allow => "allow",
        _ => "allow",
    };

    // Forward (possibly masked) request
    let mut upstream_req = Request::from_parts(parts, Full::new(final_body));
    if let Ok(parsed) = uri.to_string().parse() {
        *upstream_req.uri_mut() = parsed;
    }
    let response = forward_request(upstream_req).await?;
    let status_code = response.status().as_u16();
    send_event(
        &event_tx,
        build_proxy_event(
            action_str,
            &host,
            findings.len() as u32,
            categories,
            body_size,
            elapsed_ms(request_started),
            status_code,
        ),
    );
    Ok(response)
}

async fn forward_request(req: Request<Full<Bytes>>) -> Result<Response<BoxBody>, hyper::Error> {
    let connector = hyper_util::client::legacy::connect::HttpConnector::new();
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector);

    let resp: Result<Response<Incoming>, _> = client.request(req).await;
    match resp {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            let body_bytes: Bytes = body.collect().await?.to_bytes();
            Ok(Response::from_parts(parts, full_body(body_bytes)))
        }
        Err(e) => {
            tracing::error!(target: "eidra::proxy", error = %e, "upstream request failed");
            let mut resp = Response::new(full_body(format!("Upstream error: {}", e)));
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            Ok(resp)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn log_decision(
    audit: &AuditStore,
    event_tx: &EventSender,
    audit_action: ActionTaken,
    action_label: &str,
    destination: &str,
    findings_count: u32,
    categories: Vec<String>,
    data_size_bytes: u64,
    latency_ms: u64,
    status_code: u16,
) {
    let summary = serde_json::to_string(&categories).unwrap_or_default();
    let event = AuditEvent::new(
        EventType::AiRequest,
        audit_action,
        destination,
        findings_count,
        summary,
        data_size_bytes,
    );
    let _ = audit.log_event(&event);
    send_event(
        event_tx,
        build_proxy_event(
            action_label,
            destination,
            findings_count,
            categories,
            data_size_bytes,
            latency_ms,
            status_code,
        ),
    );
}

fn escalate_response(findings_count: usize, categories: Vec<String>) -> Response<BoxBody> {
    let body = serde_json::json!({
        "error": "manual_review_required",
        "message": "Request requires approval before leaving this device",
        "findings_count": findings_count,
        "categories": categories,
    });
    let mut resp = Response::new(full_body(body.to_string()));
    *resp.status_mut() = StatusCode::FORBIDDEN;
    resp
}

fn body_too_large_response(
    audit: &AuditStore,
    event_tx: &EventSender,
    destination: &str,
    body_size: u64,
    max_body_size: usize,
    latency_ms: u64,
) -> Response<BoxBody> {
    log_decision(
        audit,
        event_tx,
        ActionTaken::Custom("body_too_large".to_string()),
        "block",
        destination,
        0,
        vec!["body_too_large".to_string()],
        body_size,
        latency_ms,
        StatusCode::PAYLOAD_TOO_LARGE.as_u16(),
    );

    let body = serde_json::json!({
        "error": "request_too_large",
        "message": format!(
            "Request exceeded Eidra body size limit ({} bytes > {} bytes)",
            body_size, max_body_size
        ),
        "max_body_size": max_body_size,
        "request_size": body_size,
    });
    let mut resp = Response::new(full_body(body.to_string()));
    *resp.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
    resp
}

#[allow(clippy::too_many_arguments)]
async fn route_to_local_llm(
    local_router: Option<Arc<OllamaRouter>>,
    destination: &str,
    request_path: &str,
    body_str: &str,
    audit: &AuditStore,
    event_tx: &EventSender,
    findings_count: u32,
    categories: Vec<String>,
    data_size_bytes: u64,
    request_started: Instant,
) -> Result<Response<BoxBody>, hyper::Error> {
    if !supports_local_chat_route(request_path) {
        log_decision(
            audit,
            event_tx,
            ActionTaken::Custom("route_local_unsupported".to_string()),
            "block",
            destination,
            findings_count,
            categories.clone(),
            data_size_bytes,
            elapsed_ms(request_started),
            StatusCode::NOT_IMPLEMENTED.as_u16(),
        );
        let body = serde_json::json!({
            "error": "local_route_unsupported",
            "message": "Local routing currently supports OpenAI-compatible chat completion requests only",
            "request_path": request_path,
            "findings_count": findings_count,
            "categories": categories,
        });
        let mut resp = Response::new(full_body(body.to_string()));
        *resp.status_mut() = StatusCode::NOT_IMPLEMENTED;
        return Ok(resp);
    }

    let Some(local_router) = local_router else {
        log_decision(
            audit,
            event_tx,
            ActionTaken::Custom("route_local_unavailable".to_string()),
            "block",
            destination,
            findings_count,
            categories.clone(),
            data_size_bytes,
            elapsed_ms(request_started),
            StatusCode::BAD_GATEWAY.as_u16(),
        );
        let body = serde_json::json!({
            "error": "local_route_unavailable",
            "message": "Policy requested local routing, but local_llm is not enabled in Eidra config",
            "findings_count": findings_count,
            "categories": categories,
        });
        let mut resp = Response::new(full_body(body.to_string()));
        *resp.status_mut() = StatusCode::BAD_GATEWAY;
        return Ok(resp);
    };

    match local_router.route(body_str).await {
        Ok(response_body) => {
            log_decision(
                audit,
                event_tx,
                ActionTaken::Custom("route_local".to_string()),
                "route",
                destination,
                findings_count,
                categories,
                data_size_bytes,
                elapsed_ms(request_started),
                StatusCode::OK.as_u16(),
            );

            let mut resp = Response::new(full_body(response_body));
            *resp.status_mut() = StatusCode::OK;
            resp.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            );
            resp.headers_mut().insert(
                hyper::header::HeaderName::from_static("x-eidra-route"),
                hyper::header::HeaderValue::from_static("local_llm"),
            );
            Ok(resp)
        }
        Err(err) => {
            tracing::warn!(
                target: "eidra::proxy",
                destination = %destination,
                error = %err,
                "local LLM routing failed"
            );
            log_decision(
                audit,
                event_tx,
                ActionTaken::Custom("route_local_failed".to_string()),
                "block",
                destination,
                findings_count,
                categories.clone(),
                data_size_bytes,
                elapsed_ms(request_started),
                StatusCode::BAD_GATEWAY.as_u16(),
            );
            let body = serde_json::json!({
                "error": "local_route_failed",
                "message": err.to_string(),
                "findings_count": findings_count,
                "categories": categories,
            });
            let mut resp = Response::new(full_body(body.to_string()));
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            Ok(resp)
        }
    }
}

fn supports_local_chat_route(request_path: &str) -> bool {
    request_path.ends_with("/chat/completions")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_openai_chat_completion_routes() {
        assert!(supports_local_chat_route("/v1/chat/completions"));
        assert!(supports_local_chat_route("/chat/completions"));
        assert!(!supports_local_chat_route("/v1/messages"));
        assert!(!supports_local_chat_route("/v1/responses"));
    }
}
