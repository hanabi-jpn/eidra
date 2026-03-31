use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::runtime::{
    build_mcp_gateway_config, effective_mcp_listen, effective_proxy_listen, load_app_config,
    runtime_paths,
};

#[derive(Debug, Serialize)]
struct SetupArtifact {
    name: String,
    path: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct SetupPlan {
    target: String,
    title: String,
    steps: Vec<String>,
    environment: HashMap<String, String>,
    notes: Vec<String>,
    artifacts: Vec<SetupArtifact>,
}

pub async fn run(target: Option<String>, write: bool, json: bool) -> Result<()> {
    let paths = runtime_paths()?;
    let config = load_app_config(&paths)?;
    let proxy_listen = effective_proxy_listen(&config, None);
    let mcp_config = build_mcp_gateway_config(&config);
    let mcp_listen = effective_mcp_listen(&config, None);

    let target = target.unwrap_or_else(|| "shell".to_string());
    let target = normalize_target(&target);

    if target == "list" {
        let targets = supported_targets();
        if json {
            println!("{}", serde_json::to_string_pretty(&targets)?);
        } else {
            println!("Supported setup targets");
            println!();
            for (name, detail) in targets {
                println!("  {:<13} {}", name, detail);
            }
        }
        return Ok(());
    }

    let mut plan = build_setup_plan(
        &target,
        &paths,
        &proxy_listen,
        &mcp_listen,
        mcp_config.server_whitelist.len(),
    )?;

    if write {
        plan.artifacts = write_setup_artifacts(&paths, &plan)?;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    println!("{}", plan.title);
    println!();
    for (idx, step) in plan.steps.iter().enumerate() {
        println!("{}. {}", idx + 1, step);
    }

    if !plan.environment.is_empty() {
        println!();
        println!("Environment");
        for (key, value) in &plan.environment {
            println!("  export {}={}", key, value);
        }
    }

    if !plan.notes.is_empty() {
        println!();
        println!("Notes");
        for note in &plan.notes {
            println!("  - {}", note);
        }
    }

    if !plan.artifacts.is_empty() {
        println!();
        println!("Generated Artifacts");
        for artifact in &plan.artifacts {
            println!("  - {}: {}", artifact.name, artifact.path);
        }
    }

    Ok(())
}

fn build_setup_plan(
    target: &str,
    paths: &crate::runtime::RuntimePaths,
    proxy_listen: &str,
    mcp_listen: &str,
    mcp_server_count: usize,
) -> Result<SetupPlan> {
    let proxy_url = format!("http://{}", proxy_listen);
    let mcp_url = format!("http://{}", mcp_listen);

    let env_map = HashMap::from([
        ("HTTP_PROXY".to_string(), proxy_url.clone()),
        ("HTTPS_PROXY".to_string(), proxy_url.clone()),
        ("http_proxy".to_string(), proxy_url.clone()),
        ("https_proxy".to_string(), proxy_url.clone()),
        ("NO_PROXY".to_string(), "localhost,127.0.0.1".to_string()),
    ]);

    let plan = match target {
        "shell" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for shell / generic local tools".to_string(),
            steps: vec![
                format!(
                    "Run `eidra init` and trust the CA certificate at {}.",
                    paths.ca_cert_path.display()
                ),
                "Start the proxy with `eidra start`.".to_string(),
                "Export the environment variables below before launching your AI tool."
                    .to_string(),
            ],
            environment: env_map,
            notes: vec![
                "Keep `NO_PROXY=localhost,127.0.0.1` so local tools still work.".to_string(),
                "Use `eidra doctor` to confirm CA, proxy, and local LLM readiness.".to_string(),
            ],
            artifacts: Vec::new(),
        },
        "cursor" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for Cursor".to_string(),
            steps: vec![
                format!(
                    "Run `eidra init` and trust the CA certificate at {}.",
                    paths.ca_cert_path.display()
                ),
                "Start the proxy with `eidra start`.".to_string(),
                "Launch Cursor from a shell that exports the environment variables below."
                    .to_string(),
            ],
            environment: env_map,
            notes: vec![
                "Cursor works best when it inherits proxy variables from the shell that launched it."
                    .to_string(),
                "Use `eidra setup cursor --write` to generate a reusable env script."
                    .to_string(),
            ],
            artifacts: Vec::new(),
        },
        "claude-code" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for Claude Code".to_string(),
            steps: vec![
                format!(
                    "Run `eidra init` and trust the CA certificate at {}.",
                    paths.ca_cert_path.display()
                ),
                "Start the proxy with `eidra start`.".to_string(),
                "Launch Claude Code from a shell that exports the environment variables below."
                    .to_string(),
                "If you use MCP, start the firewall separately with `eidra gateway`."
                    .to_string(),
            ],
            environment: env_map,
            notes: vec![format!(
                "Point MCP clients at {} once the gateway is enabled.",
                mcp_url
            )],
            artifacts: Vec::new(),
        },
        "codex" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for Codex CLI".to_string(),
            steps: vec![
                format!(
                    "Run `eidra init` and trust the CA certificate at {}.",
                    paths.ca_cert_path.display()
                ),
                "Start the proxy with `eidra start`.".to_string(),
                "Launch Codex from a shell that exports the environment variables below."
                    .to_string(),
                "If you use MCP with Codex, start the firewall separately with `eidra gateway`."
                    .to_string(),
            ],
            environment: env_map,
            notes: vec![
                "Codex works best when it inherits proxy variables from the shell or wrapper that launched it."
                    .to_string(),
                format!(
                    "Point MCP-aware tools at {} once the gateway is enabled.",
                    mcp_url
                ),
                "Use `eidra setup codex --write` to generate a reusable env script."
                    .to_string(),
            ],
            artifacts: Vec::new(),
        },
        "openai-sdk" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for OpenAI-compatible SDKs".to_string(),
            steps: vec![
                "Start Eidra with `eidra start`.".to_string(),
                "Export the environment variables below before running your application."
                    .to_string(),
                "Keep your existing OpenAI client code unchanged.".to_string(),
            ],
            environment: env_map,
            notes: vec![
                "If policy routes sensitive prompts locally, Eidra responds with OpenAI-compatible chat completions."
                    .to_string(),
            ],
            artifacts: Vec::new(),
        },
        "anthropic-sdk" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for Anthropic-compatible SDKs".to_string(),
            steps: vec![
                "Start Eidra with `eidra start`.".to_string(),
                "Export the environment variables below before running your application."
                    .to_string(),
            ],
            environment: env_map,
            notes: vec![
                "Anthropic traffic is scanned, masked, and blocked through the proxy."
                    .to_string(),
                "Local routing is currently optimized for OpenAI-compatible chat requests."
                    .to_string(),
            ],
            artifacts: Vec::new(),
        },
        "github-actions" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for GitHub Actions".to_string(),
            steps: vec![
                "Add the generated workflow snippet to your job or composite action.".to_string(),
                "Start Eidra before AI-assisted or networked build steps.".to_string(),
                "Let subsequent steps inherit `HTTP_PROXY` and `HTTPS_PROXY` from `$GITHUB_ENV`."
                    .to_string(),
            ],
            environment: HashMap::new(),
            notes: vec![
                "Use `eidra setup github-actions --write` to generate a reusable snippet."
                    .to_string(),
            ],
            artifacts: Vec::new(),
        },
        "mcp" => SetupPlan {
            target: target.to_string(),
            title: "Eidra setup for MCP firewall".to_string(),
            steps: vec![
                "Configure `mcp_gateway.server_whitelist` in your Eidra config.".to_string(),
                "Start the gateway with `eidra gateway`.".to_string(),
                format!("Point MCP clients at {}.", mcp_url),
            ],
            environment: HashMap::from([("EIDRA_MCP_GATEWAY_URL".to_string(), mcp_url)]),
            notes: vec![if mcp_server_count == 0 {
                format!(
                    "No MCP servers are configured yet. Add them in {}.",
                    paths.config_path.display()
                )
            } else {
                format!("{} MCP server(s) are already configured.", mcp_server_count)
            }],
            artifacts: Vec::new(),
        },
        other => {
            anyhow::bail!(
                "unknown setup target '{}'. Run `eidra setup list` to see supported targets.",
                other
            );
        }
    };

    Ok(plan)
}

fn write_setup_artifacts(
    paths: &crate::runtime::RuntimePaths,
    plan: &SetupPlan,
) -> Result<Vec<SetupArtifact>> {
    let base_dir = paths.eidra_dir.join("generated").join(&plan.target);
    std::fs::create_dir_all(&base_dir)
        .with_context(|| format!("failed to create {}", base_dir.display()))?;

    let mut artifacts = Vec::new();

    let readme_path = base_dir.join("README.md");
    std::fs::write(&readme_path, render_plan_markdown(plan))?;
    artifacts.push(SetupArtifact {
        name: "README".to_string(),
        path: readme_path.display().to_string(),
        description: "Human-readable setup instructions".to_string(),
    });

    if !plan.environment.is_empty() {
        let env_path = base_dir.join("eidra.env");
        std::fs::write(&env_path, render_env_exports(&plan.environment))?;
        artifacts.push(SetupArtifact {
            name: "Environment exports".to_string(),
            path: env_path.display().to_string(),
            description: "Shell-ready proxy environment variables".to_string(),
        });
    }

    if plan.target == "github-actions" {
        let workflow_path = base_dir.join("eidra-github-actions.yml");
        std::fs::write(&workflow_path, render_github_actions_snippet(paths))?;
        artifacts.push(SetupArtifact {
            name: "GitHub Actions snippet".to_string(),
            path: workflow_path.display().to_string(),
            description: "Reusable workflow fragment for CI".to_string(),
        });
    }

    if plan.target == "mcp" {
        let mcp_path = base_dir.join("mcp-gateway.json");
        std::fs::write(
            &mcp_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "gateway_url": plan.environment.get("EIDRA_MCP_GATEWAY_URL"),
                "notes": plan.notes,
            }))?,
        )?;
        artifacts.push(SetupArtifact {
            name: "MCP gateway metadata".to_string(),
            path: mcp_path.display().to_string(),
            description: "Machine-readable MCP gateway hint".to_string(),
        });
    }

    Ok(artifacts)
}

fn render_plan_markdown(plan: &SetupPlan) -> String {
    let mut body = format!("# {}\n\n", plan.title);

    if !plan.steps.is_empty() {
        body.push_str("## Steps\n\n");
        for (idx, step) in plan.steps.iter().enumerate() {
            body.push_str(&format!("{}. {}\n", idx + 1, step));
        }
        body.push('\n');
    }

    if !plan.environment.is_empty() {
        body.push_str("## Environment\n\n```bash\n");
        body.push_str(&render_env_exports(&plan.environment));
        body.push_str("```\n\n");
    }

    if !plan.notes.is_empty() {
        body.push_str("## Notes\n\n");
        for note in &plan.notes {
            body.push_str(&format!("- {}\n", note));
        }
    }

    body
}

fn render_env_exports(environment: &HashMap<String, String>) -> String {
    let mut keys: Vec<_> = environment.keys().collect();
    keys.sort();

    let mut lines = Vec::new();
    for key in keys {
        let value = environment
            .get(key)
            .expect("environment key should be present");
        lines.push(format!("export {}={}", key, value));
    }
    lines.join("\n") + "\n"
}

fn render_github_actions_snippet(paths: &crate::runtime::RuntimePaths) -> String {
    format!(
        "steps:\n  - name: Start Eidra\n    run: |\n      cargo install --path crates/eidra-core\n      eidra init\n      nohup eidra start > /tmp/eidra.log 2>&1 &\n      echo HTTP_PROXY=http://127.0.0.1:8080 >> $GITHUB_ENV\n      echo HTTPS_PROXY=http://127.0.0.1:8080 >> $GITHUB_ENV\n      echo NO_PROXY=localhost,127.0.0.1 >> $GITHUB_ENV\n      echo EIDRA_CA_PATH={} >> $GITHUB_ENV\n",
        paths.ca_cert_path.display()
    )
}

fn supported_targets() -> Vec<(&'static str, &'static str)> {
    vec![
        ("shell", "Generic local shell / proxy setup"),
        ("cursor", "Cursor launched from a proxied shell"),
        (
            "claude-code",
            "Claude Code with proxy + optional MCP firewall",
        ),
        ("codex", "Codex CLI launched from a proxied shell"),
        ("openai-sdk", "OpenAI-compatible SDK traffic through Eidra"),
        (
            "anthropic-sdk",
            "Anthropic-compatible SDK traffic through Eidra",
        ),
        ("github-actions", "CI setup using GitHub Actions"),
        ("mcp", "MCP firewall gateway"),
    ]
}

fn normalize_target(target: &str) -> String {
    match target.trim().to_ascii_lowercase().as_str() {
        "" => "shell".to_string(),
        "generic" => "shell".to_string(),
        "claude" => "claude-code".to_string(),
        "codex-cli" => "codex".to_string(),
        "openai" => "openai-sdk".to_string(),
        "anthropic" => "anthropic-sdk".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_target_maps_aliases() {
        assert_eq!(normalize_target(""), "shell");
        assert_eq!(normalize_target("generic"), "shell");
        assert_eq!(normalize_target("claude"), "claude-code");
        assert_eq!(normalize_target("codex-cli"), "codex");
        assert_eq!(normalize_target("openai"), "openai-sdk");
        assert_eq!(normalize_target("anthropic"), "anthropic-sdk");
    }

    #[test]
    fn render_env_exports_sorts_keys() {
        let env = HashMap::from([
            ("HTTPS_PROXY".to_string(), "https://example".to_string()),
            ("HTTP_PROXY".to_string(), "http://example".to_string()),
        ]);
        let rendered = render_env_exports(&env);
        assert!(
            rendered.starts_with("export HTTPS_PROXY=")
                || rendered.starts_with("export HTTP_PROXY=")
        );
        assert!(rendered.contains("export HTTP_PROXY=http://example"));
        assert!(rendered.contains("export HTTPS_PROXY=https://example"));
    }
}
