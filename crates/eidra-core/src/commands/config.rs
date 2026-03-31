use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::runtime::{
    build_local_router, build_mcp_gateway_config, build_scanner, load_app_config,
    load_policy_config, runtime_paths,
};

#[derive(Debug, Serialize)]
struct ConfigPaths {
    eidra_dir: String,
    config_path: String,
    policy_path: String,
    audit_db_path: String,
    ca_cert_path: String,
    ca_key_path: String,
}

#[derive(Debug, Serialize)]
struct ConfigShowReport {
    paths: ConfigPaths,
    config_exists: bool,
    policy_exists: bool,
    config_content: Option<String>,
    policy_content: Option<String>,
}

#[derive(Debug, Serialize)]
struct ValidationReport {
    config_path: String,
    config_exists: bool,
    policy_path: String,
    policy_exists: bool,
    policy_rules: usize,
    scanner_classifiers: usize,
    proxy_listen: String,
    proxy_max_body_size: usize,
    local_llm_enabled: bool,
    local_llm_endpoint: Option<String>,
    local_llm_model: Option<String>,
    mcp_gateway_enabled: bool,
    mcp_gateway_listen: String,
    mcp_server_count: usize,
}

pub async fn run(action: Option<String>, json: bool) -> Result<()> {
    let paths = runtime_paths()?;

    match action.as_deref() {
        Some("show") | None => show_config(&paths, json),
        Some("path") => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "eidra_dir": paths.eidra_dir.display().to_string()
                    }))?
                );
            } else {
                println!("{}", paths.eidra_dir.display());
            }
            Ok(())
        }
        Some("edit") => {
            ensure_runtime_dir(&paths)?;
            open_in_editor(&paths.config_path)?;
            Ok(())
        }
        Some("edit-policy") => {
            ensure_runtime_dir(&paths)?;
            open_in_editor(&paths.policy_path)?;
            Ok(())
        }
        Some("reset") => {
            ensure_runtime_dir(&paths)?;
            let default_config = include_str!("../../../../config/default.yaml");
            let default_policy = include_str!("../../../../config/policies/default.yaml");
            std::fs::write(&paths.config_path, default_config)?;
            std::fs::write(&paths.policy_path, default_policy)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "ok",
                        "message": "Configuration reset to defaults.",
                        "config_path": paths.config_path.display().to_string(),
                        "policy_path": paths.policy_path.display().to_string(),
                    }))?
                );
            } else {
                println!("Configuration reset to defaults.");
            }
            Ok(())
        }
        Some("validate") => validate_config(&paths, json),
        Some(other) => {
            println!("Unknown config action: {}", other);
            println!();
            println!("Usage:");
            println!("  eidra config              Show current configuration");
            println!("  eidra config show         Show current configuration");
            println!("  eidra config path         Print config directory path");
            println!("  eidra config edit         Open config.yaml in $EDITOR");
            println!("  eidra config edit-policy  Open policy.yaml in $EDITOR");
            println!("  eidra config reset        Reset to default configuration");
            println!("  eidra config validate     Parse and validate config + policy");
            println!("  eidra config --json       Emit supported output as JSON");
            Ok(())
        }
    }
}

fn show_config(paths: &crate::runtime::RuntimePaths, json: bool) -> Result<()> {
    let report = ConfigShowReport {
        paths: config_paths(paths),
        config_exists: paths.config_path.exists(),
        policy_exists: paths.policy_path.exists(),
        config_content: read_if_exists(&paths.config_path)?,
        policy_content: read_if_exists(&paths.policy_path)?,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Eidra Configuration");
    println!("===================");
    println!();
    println!("Config dir:  {}", report.paths.eidra_dir);
    println!("Config file: {}", report.paths.config_path);
    println!("Policy file: {}", report.paths.policy_path);
    println!("Audit DB:    {}", report.paths.audit_db_path);
    println!("CA cert:     {}", report.paths.ca_cert_path);
    println!("CA key:      {}", report.paths.ca_key_path);
    println!();

    if let Some(content) = report.config_content.as_ref() {
        println!("--- config.yaml ---");
        println!("{}", content);
    } else {
        println!("Config file not found. Run `eidra init` first.");
    }

    if let Some(content) = report.policy_content.as_ref() {
        println!("--- policy.yaml ---");
        println!("{}", content);
    }

    Ok(())
}

fn validate_config(paths: &crate::runtime::RuntimePaths, json: bool) -> Result<()> {
    let config = load_app_config(paths)?;
    let policy = load_policy_config(paths)?;
    let scanner = build_scanner(&config, true)?;
    let local_router = build_local_router(&config)?;
    let mcp_config = build_mcp_gateway_config(&config);

    let report = ValidationReport {
        config_path: paths.config_path.display().to_string(),
        config_exists: paths.config_path.exists(),
        policy_path: paths.policy_path.display().to_string(),
        policy_exists: paths.policy_path.exists(),
        policy_rules: policy.rules.len(),
        scanner_classifiers: scanner.classifier_count(),
        proxy_listen: config.proxy.listen.clone(),
        proxy_max_body_size: config.proxy.max_body_size,
        local_llm_enabled: local_router.is_some(),
        local_llm_endpoint: local_router
            .as_ref()
            .map(|router| router.endpoint().to_string()),
        local_llm_model: local_router
            .as_ref()
            .map(|router| router.model().to_string()),
        mcp_gateway_enabled: config.mcp_gateway.enabled,
        mcp_gateway_listen: config.mcp_gateway.listen.clone(),
        mcp_server_count: mcp_config.server_whitelist.len(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Eidra Configuration Validation");
    println!("==============================");
    println!();
    println!(
        "config.yaml: {} ({})",
        if report.config_exists {
            "ok"
        } else {
            "missing; using built-in defaults"
        },
        report.config_path
    );
    println!(
        "policy.yaml: {} ({} rule(s))",
        if report.policy_exists {
            "ok"
        } else {
            "missing; using built-in defaults"
        },
        report.policy_rules
    );
    println!("scanner: ok ({} classifier(s))", report.scanner_classifiers);
    println!(
        "proxy: ok (listen {}, max body {} bytes)",
        report.proxy_listen, report.proxy_max_body_size
    );
    match (
        report.local_llm_endpoint.as_ref(),
        report.local_llm_model.as_ref(),
    ) {
        (Some(endpoint), Some(model)) => {
            println!(
                "local_llm: ok (provider {}, endpoint {}, default model {})",
                config.local_llm.provider, endpoint, model
            );
        }
        _ => println!("local_llm: disabled"),
    }
    if report.mcp_gateway_enabled {
        println!(
            "mcp_gateway: ok (listen {}, {} server(s))",
            report.mcp_gateway_listen, report.mcp_server_count
        );
    } else {
        println!("mcp_gateway: disabled");
    }

    Ok(())
}

fn config_paths(paths: &crate::runtime::RuntimePaths) -> ConfigPaths {
    ConfigPaths {
        eidra_dir: paths.eidra_dir.display().to_string(),
        config_path: paths.config_path.display().to_string(),
        policy_path: paths.policy_path.display().to_string(),
        audit_db_path: paths.eidra_dir.join("audit.db").display().to_string(),
        ca_cert_path: paths.ca_cert_path.display().to_string(),
        ca_key_path: paths.ca_key_path.display().to_string(),
    }
}

fn read_if_exists(path: &Path) -> Result<Option<String>> {
    if path.exists() {
        Ok(Some(std::fs::read_to_string(path)?))
    } else {
        Ok(None)
    }
}

fn ensure_runtime_dir(paths: &crate::runtime::RuntimePaths) -> Result<()> {
    std::fs::create_dir_all(&paths.eidra_dir)
        .with_context(|| format!("failed to create {}", paths.eidra_dir.display()))
}

fn open_in_editor(path: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(&editor).arg(path).status()?;
    if !status.success() {
        anyhow::bail!("editor exited with non-zero status");
    }
    Ok(())
}
