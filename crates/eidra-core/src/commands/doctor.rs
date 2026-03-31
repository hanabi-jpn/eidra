use std::time::Duration;

use anyhow::Result;
use serde::Serialize;

use crate::runtime::{
    build_local_router, build_mcp_gateway_config, build_scanner, effective_mcp_listen,
    effective_proxy_listen, expand_tilde, load_app_config, load_policy_config, path_status,
    runtime_paths,
};

#[derive(Debug, Serialize)]
struct DoctorCheck {
    label: String,
    status: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    checks: Vec<DoctorCheck>,
    next_steps: Vec<String>,
}

pub async fn run(json: bool) -> Result<()> {
    let report = build_report().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Eidra Doctor");
    println!("============");
    println!();

    for check in &report.checks {
        println!("[{:<5}] {:<16} {}", check.status, check.label, check.detail);
    }

    println!();
    println!("Next Steps");
    for (idx, step) in report.next_steps.iter().enumerate() {
        println!("{}. {}", idx + 1, step);
    }

    Ok(())
}

async fn build_report() -> Result<DoctorReport> {
    let paths = runtime_paths()?;
    let mut checks = vec![
        DoctorCheck {
            label: "config directory".to_string(),
            status: "ok".to_string(),
            detail: paths.eidra_dir.display().to_string(),
        },
        DoctorCheck {
            label: "config.yaml".to_string(),
            status: if paths.config_path.exists() {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: path_status(&paths.config_path).to_string(),
        },
        DoctorCheck {
            label: "policy.yaml".to_string(),
            status: if paths.policy_path.exists() {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: path_status(&paths.policy_path).to_string(),
        },
        DoctorCheck {
            label: "CA certificate".to_string(),
            status: if paths.ca_cert_path.exists() {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: path_status(&paths.ca_cert_path).to_string(),
        },
        DoctorCheck {
            label: "CA private key".to_string(),
            status: if paths.ca_key_path.exists() {
                "ok".to_string()
            } else {
                "warn".to_string()
            },
            detail: path_status(&paths.ca_key_path).to_string(),
        },
    ];

    let config = match load_app_config(&paths) {
        Ok(config) => {
            checks.push(DoctorCheck {
                label: "config parse".to_string(),
                status: "ok".to_string(),
                detail: if paths.config_path.exists() {
                    "configuration loaded".to_string()
                } else {
                    "using built-in defaults".to_string()
                },
            });
            Some(config)
        }
        Err(err) => {
            checks.push(DoctorCheck {
                label: "config parse".to_string(),
                status: "error".to_string(),
                detail: err.to_string(),
            });
            None
        }
    };

    match load_policy_config(&paths) {
        Ok(policy) => {
            let detail = if paths.policy_path.exists() {
                format!("{} rule(s) loaded", policy.rules.len())
            } else {
                format!("{} built-in rule(s) loaded", policy.rules.len())
            };
            checks.push(DoctorCheck {
                label: "policy parse".to_string(),
                status: "ok".to_string(),
                detail,
            });
        }
        Err(err) => {
            checks.push(DoctorCheck {
                label: "policy parse".to_string(),
                status: "error".to_string(),
                detail: err.to_string(),
            });
        }
    }

    if let Some(config) = config.as_ref() {
        match build_scanner(config, true) {
            Ok(scanner) => {
                let detail = format!(
                    "{} classifier(s), scan {}",
                    scanner.classifier_count(),
                    if config.scan.enabled {
                        "enabled"
                    } else {
                        "disabled for proxy"
                    }
                );
                checks.push(DoctorCheck {
                    label: "scan engine".to_string(),
                    status: "ok".to_string(),
                    detail,
                });
            }
            Err(err) => {
                checks.push(DoctorCheck {
                    label: "scan engine".to_string(),
                    status: "error".to_string(),
                    detail: err.to_string(),
                });
            }
        }

        checks.push(DoctorCheck {
            label: "proxy runtime".to_string(),
            status: "ok".to_string(),
            detail: format!(
                "listen {}, max body {} bytes",
                effective_proxy_listen(config, None),
                config.proxy.max_body_size
            ),
        });

        if config.audit.enabled {
            let audit_path = expand_tilde(&config.audit.db_path)?;
            checks.push(DoctorCheck {
                label: "audit log".to_string(),
                status: "ok".to_string(),
                detail: audit_path.display().to_string(),
            });
        } else {
            checks.push(DoctorCheck {
                label: "audit log".to_string(),
                status: "warn".to_string(),
                detail: "disabled; runtime uses in-memory storage".to_string(),
            });
        }

        if config.scan.custom_rules_path.trim().is_empty() {
            checks.push(DoctorCheck {
                label: "custom rules".to_string(),
                status: "warn".to_string(),
                detail: "disabled".to_string(),
            });
        } else {
            let custom_rules_path = expand_tilde(&config.scan.custom_rules_path)?;
            checks.push(DoctorCheck {
                label: "custom rules".to_string(),
                status: if custom_rules_path.exists() {
                    "ok".to_string()
                } else {
                    "error".to_string()
                },
                detail: custom_rules_path.display().to_string(),
            });
        }

        if config.local_llm.enabled {
            match build_local_router(config) {
                Ok(Some(router)) => {
                    let detail = format!(
                        "{} at {} ({})",
                        config.local_llm.provider,
                        router.endpoint(),
                        router.model()
                    );
                    let status = if endpoint_reachable(router.endpoint()).await {
                        "ok"
                    } else {
                        "warn"
                    };
                    let detail = if status == "ok" {
                        detail
                    } else {
                        format!("{}; endpoint unreachable", detail)
                    };
                    checks.push(DoctorCheck {
                        label: "local LLM".to_string(),
                        status: status.to_string(),
                        detail,
                    });
                }
                Ok(None) => {
                    checks.push(DoctorCheck {
                        label: "local LLM".to_string(),
                        status: "warn".to_string(),
                        detail: "disabled".to_string(),
                    });
                }
                Err(err) => {
                    checks.push(DoctorCheck {
                        label: "local LLM".to_string(),
                        status: "error".to_string(),
                        detail: err.to_string(),
                    });
                }
            }
        } else {
            checks.push(DoctorCheck {
                label: "local LLM".to_string(),
                status: "warn".to_string(),
                detail: "disabled".to_string(),
            });
        }

        if config.mcp_gateway.enabled {
            let gateway_config = build_mcp_gateway_config(config);
            checks.push(DoctorCheck {
                label: "MCP gateway".to_string(),
                status: "ok".to_string(),
                detail: format!(
                    "listen {}, {} server(s)",
                    effective_mcp_listen(config, None),
                    gateway_config.server_whitelist.len()
                ),
            });
        } else {
            checks.push(DoctorCheck {
                label: "MCP gateway".to_string(),
                status: "warn".to_string(),
                detail: "disabled".to_string(),
            });
        }
    }

    Ok(DoctorReport {
        checks,
        next_steps: vec![
            "Run `eidra init` if config, policy, or CA files are missing.".to_string(),
            "Run `eidra start` for transparent proxy protection.".to_string(),
            "Run `eidra setup shell` or `eidra setup cursor` for environment-specific wiring."
                .to_string(),
            "Run `eidra gateway` if you want MCP firewall protection.".to_string(),
            "Run `eidra scan <file>` or `eidra scan --json` to validate CI and local inputs."
                .to_string(),
        ],
    })
}

async fn endpoint_reachable(endpoint: &str) -> bool {
    let Some(authority) = endpoint_authority(endpoint) else {
        return false;
    };

    matches!(
        tokio::time::timeout(
            Duration::from_secs(2),
            tokio::net::TcpStream::connect(authority)
        )
        .await,
        Ok(Ok(_))
    )
}

fn endpoint_authority(endpoint: &str) -> Option<String> {
    let (scheme, remainder) = endpoint
        .split_once("://")
        .map_or(("http", endpoint), |(scheme, rest)| (scheme, rest));
    let authority = remainder.split('/').next()?.trim();
    if authority.is_empty() {
        return None;
    }

    if authority.contains(':') {
        Some(authority.to_string())
    } else if scheme.eq_ignore_ascii_case("https") {
        Some(format!("{authority}:443"))
    } else {
        Some(format!("{authority}:80"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_authority_extracts_host_and_port() {
        assert_eq!(
            endpoint_authority("http://localhost:11434/api/chat").as_deref(),
            Some("localhost:11434")
        );
        assert_eq!(
            endpoint_authority("https://example.com/path").as_deref(),
            Some("example.com:443")
        );
    }
}
