use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::runtime::runtime_paths;

const DEFAULT_REPO_URL: &str = "https://github.com/<your-org-or-user>/eidra";
const ABOUT_OPTION_A: &str =
    "Local-first safety filter for AI development. See what your AI tools send, then hide, stop, or route risky traffic before it leaves your machine.";
const ABOUT_OPTION_B: &str =
    "Open-source local proxy and MCP firewall for safer AI development workflows.";
const ABOUT_OPTION_C: &str =
    "A localhost safety filter for AI tools, MCP workflows, and agentic development.";
const RELEASE_SUMMARY: &str = "Eidra is an open-source local-first safety filter for AI development. Put it in front of Cursor, Claude Code, Codex, SDK workflows, or MCP tools to inspect, mask, block, or route sensitive traffic.";
const FEEDBACK_CTA: &str =
    "If you use Cursor, Claude Code, Codex, SDK workflows, or MCP tools, I would love one honest reaction on what feels clear or confusing.";
const BUILDER_CTA: &str =
    "If you are building with MCP or agent workflows, I would especially love feedback on integration paths and policy ergonomics.";
const TOPICS: &[&str] = &[
    "ai-security",
    "developer-tools",
    "local-first",
    "mcp",
    "mcp-firewall",
    "proxy",
    "privacy",
    "rust",
    "agentic-ai",
    "observability",
];

#[derive(Debug, Serialize)]
struct LaunchCheck {
    label: String,
    status: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct LaunchArtifact {
    name: String,
    path: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct SocialPreviewBrief {
    headline: String,
    subhead: String,
    support_line: String,
    visual: String,
}

#[derive(Debug, Serialize)]
struct LaunchPosts {
    x: String,
    x_ja: String,
    linkedin: String,
    warm_dm_en: String,
    warm_dm_ja: String,
}

#[derive(Debug, Serialize)]
struct LaunchPlan {
    target: String,
    title: String,
    repo_dir: String,
    repo_url: Option<String>,
    repo_slug: Option<String>,
    website: String,
    release_tag: String,
    release_title: String,
    about_options: Vec<String>,
    topics: Vec<String>,
    social_preview: SocialPreviewBrief,
    release_summary: String,
    feedback_cta: String,
    builder_cta: String,
    release_notes: String,
    discussion_announcement: String,
    posts: LaunchPosts,
    checks: Vec<LaunchCheck>,
    commands: Vec<String>,
    next_steps: Vec<String>,
    artifacts: Vec<LaunchArtifact>,
}

pub async fn run(
    target: Option<String>,
    write: bool,
    json: bool,
    repo_url: Option<String>,
    repo_dir: Option<String>,
    tag: Option<String>,
) -> Result<()> {
    let target = normalize_target(target.as_deref().unwrap_or("github"));

    if target == "list" {
        let targets = supported_targets();
        if json {
            println!("{}", serde_json::to_string_pretty(&targets)?);
        } else {
            println!("Supported launch targets");
            println!();
            for (name, detail) in targets {
                println!("  {:<8} {}", name, detail);
            }
        }
        return Ok(());
    }

    let mut plan = build_launch_plan(&target, repo_url, repo_dir, tag)?;
    plan.commands = build_commands(&plan, false);

    if write {
        plan.commands = build_commands(&plan, true);
        plan.next_steps = build_next_steps(&plan.checks, true);
        let paths = runtime_paths()?;
        plan.artifacts = write_launch_artifacts(&paths, &plan)?;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    println!("{}", plan.title);
    println!();
    println!("Repository");
    println!("  dir: {}", plan.repo_dir);
    println!(
        "  url: {}",
        plan.repo_url.as_deref().unwrap_or("<not detected>")
    );
    println!("  release: {} ({})", plan.release_tag, plan.release_title);
    println!();

    println!("Readiness");
    for check in &plan.checks {
        println!("[{:<5}] {:<18} {}", check.status, check.label, check.detail);
    }

    println!();
    println!("Topics");
    println!("  {}", plan.topics.join(", "));

    println!();
    println!("Commands");
    for command in &plan.commands {
        println!("  - {}", command);
    }

    if !plan.artifacts.is_empty() {
        println!();
        println!("Generated Artifacts");
        for artifact in &plan.artifacts {
            println!("  - {}: {}", artifact.name, artifact.path);
        }
    }

    println!();
    println!("Next Steps");
    for (idx, step) in plan.next_steps.iter().enumerate() {
        println!("{}. {}", idx + 1, step);
    }

    Ok(())
}

fn build_launch_plan(
    target: &str,
    repo_url: Option<String>,
    repo_dir: Option<String>,
    tag: Option<String>,
) -> Result<LaunchPlan> {
    if target != "github" {
        bail!(
            "unknown launch target '{}'. Run `eidra launch list` to see supported targets.",
            target
        );
    }

    let repo_dir = resolve_repo_dir(repo_dir)?;
    let git_repo = is_git_worktree(&repo_dir);
    let detected_repo_url = repo_url
        .map(|value| normalize_repo_url(&value))
        .or_else(|| detect_origin_url(&repo_dir))
        .or_else(|| detect_workspace_repository(&repo_dir));
    let repo_slug = detected_repo_url.as_deref().and_then(repo_slug_from_url);
    let website = detected_repo_url
        .clone()
        .unwrap_or_else(|| DEFAULT_REPO_URL.to_string());
    let release_tag = tag.unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")));
    let release_title = format!(
        "{} — Local-First Trust Layer for AI Development",
        release_tag
    );

    let about_options = vec![
        ABOUT_OPTION_A.to_string(),
        ABOUT_OPTION_B.to_string(),
        ABOUT_OPTION_C.to_string(),
    ];
    let topics = TOPICS.iter().map(|topic| (*topic).to_string()).collect();
    let social_preview = SocialPreviewBrief {
        headline: "Eidra".to_string(),
        subhead: "Local-first trust layer for AI development".to_string(),
        support_line: "Proxy + MCP firewall + live dashboard".to_string(),
        visual: "Use the TUI dashboard screenshot or a clean tool -> Eidra -> cloud/local diagram."
            .to_string(),
    };
    let release_notes = render_release_notes(&release_tag);
    let discussion_announcement = render_discussion_announcement();
    let posts = LaunchPosts {
        x: render_x_post(&website),
        x_ja: render_x_post_ja(&website),
        linkedin: render_linkedin_post(),
        warm_dm_en: render_warm_dm_en(&website),
        warm_dm_ja: render_warm_dm_ja(&website),
    };

    let checks = build_checks(
        &repo_dir,
        git_repo,
        detected_repo_url.as_deref(),
        repo_slug.as_deref(),
        &release_tag,
    );
    let next_steps = build_next_steps(&checks, false);

    Ok(LaunchPlan {
        target: target.to_string(),
        title: "Eidra launch automation for GitHub".to_string(),
        repo_dir: repo_dir.display().to_string(),
        repo_url: detected_repo_url,
        repo_slug,
        website,
        release_tag,
        release_title,
        about_options,
        topics,
        social_preview,
        release_summary: RELEASE_SUMMARY.to_string(),
        feedback_cta: FEEDBACK_CTA.to_string(),
        builder_cta: BUILDER_CTA.to_string(),
        release_notes,
        discussion_announcement,
        posts,
        checks,
        commands: Vec::new(),
        next_steps,
        artifacts: Vec::new(),
    })
}

fn write_launch_artifacts(
    paths: &crate::runtime::RuntimePaths,
    plan: &LaunchPlan,
) -> Result<Vec<LaunchArtifact>> {
    let base_dir = paths
        .eidra_dir
        .join("generated")
        .join("launch")
        .join(&plan.target);
    std::fs::create_dir_all(&base_dir)
        .with_context(|| format!("failed to create {}", base_dir.display()))?;

    let mut artifacts = Vec::new();

    write_artifact(
        &mut artifacts,
        base_dir.join("README.md"),
        "README",
        "Human-readable GitHub launch plan",
        render_launch_markdown(plan),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("ABOUT_OPTION_A.txt"),
        "About text A",
        "Recommended GitHub About description",
        format!("{}\n", ABOUT_OPTION_A),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("ABOUT_OPTION_B.txt"),
        "About text B",
        "Alternative GitHub About description",
        format!("{}\n", ABOUT_OPTION_B),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("ABOUT_OPTION_C.txt"),
        "About text C",
        "Alternative GitHub About description",
        format!("{}\n", ABOUT_OPTION_C),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("WEBSITE.txt"),
        "Website",
        "Suggested GitHub repository website field",
        format!("{}\n", plan.website),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("TOPICS.txt"),
        "Topics",
        "Suggested GitHub repository topics",
        render_topics(&plan.topics),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("SOCIAL_PREVIEW_BRIEF.md"),
        "Social preview brief",
        "Brief for creating the GitHub social preview image",
        render_social_preview_brief(&plan.social_preview),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("RELEASE_TITLE.txt"),
        "Release title",
        "Suggested GitHub release title",
        format!("{}\n", plan.release_title),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("RELEASE_SUMMARY.txt"),
        "Release summary",
        "Short release summary for GitHub and social posts",
        format!("{}\n", plan.release_summary),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("RELEASE_NOTES.md"),
        "Release notes",
        "Ready-to-paste GitHub release notes",
        plan.release_notes.clone(),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("DISCUSSION_ANNOUNCEMENT.md"),
        "Discussion announcement",
        "Pinned GitHub Discussion announcement copy",
        plan.discussion_announcement.clone(),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("X_POST.txt"),
        "X post",
        "Launch-day post for X",
        format!("{}\n", plan.posts.x),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("X_POST_JA.txt"),
        "Japanese X post",
        "Launch-day post for Japanese audience",
        format!("{}\n", plan.posts.x_ja),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("LINKEDIN_POST.txt"),
        "LinkedIn post",
        "Launch-day post for LinkedIn",
        format!("{}\n", plan.posts.linkedin),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("WARM_DM_EN.txt"),
        "Warm DM (EN)",
        "Short warm DM for builders or creators",
        format!("{}\n", plan.posts.warm_dm_en),
    )?;
    write_artifact(
        &mut artifacts,
        base_dir.join("WARM_DM_JA.txt"),
        "Warm DM (JA)",
        "Short warm DM for Japanese builders or creators",
        format!("{}\n", plan.posts.warm_dm_ja),
    )?;

    let commands_path = base_dir.join("GITHUB_COMMANDS.md");
    let script_path = base_dir.join("apply-with-gh.sh");
    write_artifact(
        &mut artifacts,
        commands_path.clone(),
        "GitHub commands",
        "Command reference for applying the launch kit with gh",
        render_commands_markdown(plan, &script_path),
    )?;
    write_artifact(
        &mut artifacts,
        script_path.clone(),
        "Apply script",
        "Script that applies GitHub metadata and creates the first release",
        render_apply_script(plan, &base_dir),
    )?;
    make_executable(&script_path)?;

    Ok(artifacts)
}

fn write_artifact(
    artifacts: &mut Vec<LaunchArtifact>,
    path: PathBuf,
    name: &str,
    description: &str,
    content: String,
) -> Result<()> {
    std::fs::write(&path, content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    artifacts.push(LaunchArtifact {
        name: name.to_string(),
        path: path.display().to_string(),
        description: description.to_string(),
    });
    Ok(())
}

fn render_launch_markdown(plan: &LaunchPlan) -> String {
    let mut body = format!("# {}\n\n", plan.title);
    body.push_str("## Repository\n\n");
    body.push_str(&format!("- Repo dir: `{}`\n", plan.repo_dir));
    body.push_str(&format!(
        "- Repo URL: `{}`\n",
        plan.repo_url.as_deref().unwrap_or("<not detected>")
    ));
    body.push_str(&format!("- Release tag: `{}`\n", plan.release_tag));
    body.push_str(&format!("- Release title: `{}`\n\n", plan.release_title));

    body.push_str("## Readiness\n\n");
    for check in &plan.checks {
        body.push_str(&format!(
            "- `{}` `{}`: {}\n",
            check.status, check.label, check.detail
        ));
    }
    body.push('\n');

    body.push_str("## Topics\n\n");
    for topic in &plan.topics {
        body.push_str(&format!("- `{}`\n", topic));
    }
    body.push('\n');

    body.push_str("## Commands\n\n");
    for command in build_commands(plan, true) {
        body.push_str(&format!("- `{}`\n", command));
    }
    body.push('\n');

    body.push_str("## Next Steps\n\n");
    for (idx, step) in plan.next_steps.iter().enumerate() {
        body.push_str(&format!("{}. {}\n", idx + 1, step));
    }

    body
}

fn render_topics(topics: &[String]) -> String {
    let mut rendered = topics.join("\n");
    rendered.push('\n');
    rendered
}

fn render_social_preview_brief(brief: &SocialPreviewBrief) -> String {
    format!(
        "# GitHub Social Preview Brief\n\n- headline: {}\n- subhead: {}\n- support line: {}\n- visual: {}\n",
        brief.headline, brief.subhead, brief.support_line, brief.visual
    )
}

fn render_release_notes(tag: &str) -> String {
    format!(
        "## Eidra {tag}\n\nEidra is an open-source local-first safety filter for AI development.\n\nIt helps you:\n\n- see what leaves your machine\n- hide or stop risky data before it leaves\n- route sensitive requests locally when needed\n- understand what happened later\n\nThis release packages the current core workflow into a repo that people can try, review, and build on.\n\n### What is in this release\n\n- local proxy for AI traffic inspection\n- policy-based allow / mask / block / route decisions\n- MCP gateway controls\n- live terminal dashboard\n- `eidra doctor` for environment checks\n- `eidra setup` guidance for Cursor, Claude Code, Codex, SDKs, CI, and MCP\n- machine-readable scan output\n- local routing for supported OpenAI-compatible chat requests\n- beginner-friendly onboarding docs and concrete use cases\n- `eidra launch github` automation for GitHub launch assets and `gh` scripts\n\n### Good first ways to try Eidra\n\n1. Read `docs/for-everyone.md`\n2. Run `eidra doctor`\n3. Run `eidra setup codex` or swap in your tool of choice\n4. Start the dashboard and inspect a small workflow\n5. Generate a GitHub launch kit with `eidra launch github --write`\n\n### Who this is for\n\n- developers using Cursor, Claude Code, Codex, or SDK workflows\n- builders working with MCP workflows\n- teams that want a local trust boundary before heavier governance tooling\n\n### What to expect\n\nThis is an early but usable open-source release.\n\nThe product direction is clear, but some integrations and advanced routing paths are still evolving. Feedback on setup friction, policy ergonomics, and integration priorities is especially helpful.\n"
    )
}

fn render_discussion_announcement() -> String {
    "# Welcome to Eidra\n\nEidra is a local-first safety filter for AI development.\n\nIf you are here for the first time, the best starting points are:\n\n- [For Everyone](../for-everyone.md)\n- [Use Cases](../use-cases.md)\n- [For Developers](../for-developers.md)\n- [Architecture](../architecture.md)\n\nIf you try Eidra, the most useful feedback is:\n\n- where setup felt unclear\n- what workflow you wanted to protect first\n- what felt useful right away\n- what feature or integration felt missing\n\nThanks for taking a look.\n".to_string()
}

fn render_x_post(repo_url: &str) -> String {
    format!(
        "Your AI coding stack has no outbound firewall.\n\nI open-sourced Eidra: a local safety filter for Cursor, Claude Code, Codex, SDK apps, and MCP tools.\n\nSee what leaves your machine. Mask or block secrets. Route sensitive requests locally.\n\n{}",
        repo_url
    )
}

fn render_x_post_ja(repo_url: &str) -> String {
    format!(
        "Cursor / Claude Code / Codex の前に置く、安全フィルター OSS を公開しました。\n\nEidra は\n- 外に出る通信を見える化\n- secret を mask / block\n- 機密リクエストを local model に route\n\nできるツールです。\n\n{}",
        repo_url
    )
}

fn render_linkedin_post() -> String {
    "Modern AI development is no longer just chat.\n\nIt now spans Cursor, Claude Code, Codex, SDK workflows, MCP tools, and automation. That means developers need a clearer safety boundary around what leaves the machine.\n\nI’m open-sourcing Eidra as a local-first safety filter for that boundary. It combines a local proxy, policy-based traffic control, an MCP firewall, and a live dashboard so teams can see, hide, stop, or route risky AI traffic before it leaves.\n\nIf you work on AI developer tools, agent workflows, or security-adjacent infrastructure, I would love your feedback.".to_string()
}

fn render_warm_dm_en(repo_url: &str) -> String {
    format!(
        "Built an OSS called Eidra. It is a local safety filter for Cursor, Claude Code, Codex, SDK workflows, and MCP tools: you can see what leaves your machine and mask, block, or route risky traffic before it goes out. If you have 10 minutes, I would love one blunt reaction. GitHub: {}",
        repo_url
    )
}

fn render_warm_dm_ja(repo_url: &str) -> String {
    format!(
        "AIコーディングツール向けの安全フィルター OSS「Eidra」を作っています。Cursor / Claude Code / Codex / MCP の通信を見える化して、必要なら mask / block / route できます。10分だけ見てもらって、率直な違和感を1つもらえると嬉しいです。GitHub: {}",
        repo_url
    )
}

fn build_checks(
    repo_dir: &Path,
    git_repo: bool,
    repo_url: Option<&str>,
    repo_slug: Option<&str>,
    release_tag: &str,
) -> Vec<LaunchCheck> {
    let mut checks = Vec::new();

    checks.push(LaunchCheck {
        label: "repo directory".to_string(),
        status: if repo_dir.exists() { "ok" } else { "error" }.to_string(),
        detail: repo_dir.display().to_string(),
    });
    checks.push(LaunchCheck {
        label: "git worktree".to_string(),
        status: if git_repo { "ok" } else { "warn" }.to_string(),
        detail: if git_repo {
            "repository metadata detected".to_string()
        } else {
            "not running inside a git worktree; pass --repo-dir or initialize git".to_string()
        },
    });
    checks.push(LaunchCheck {
        label: "origin url".to_string(),
        status: if repo_url.is_some() { "ok" } else { "warn" }.to_string(),
        detail: repo_url
            .unwrap_or("not detected; pass --repo-url or configure remote.origin.url")
            .to_string(),
    });
    checks.push(LaunchCheck {
        label: "repo slug".to_string(),
        status: if repo_slug.is_some() { "ok" } else { "warn" }.to_string(),
        detail: repo_slug
            .unwrap_or("not inferred; use a GitHub repo URL or run apply-with-gh.sh <owner>/<repo>")
            .to_string(),
    });
    checks.push(LaunchCheck {
        label: "gh cli".to_string(),
        status: if command_success("gh", ["--version"], None) {
            "ok"
        } else {
            "warn"
        }
        .to_string(),
        detail: if command_success("gh", ["--version"], None) {
            "installed".to_string()
        } else {
            "missing; install GitHub CLI before applying launch automation".to_string()
        },
    });
    checks.push(LaunchCheck {
        label: "gh auth".to_string(),
        status: if command_success("gh", ["auth", "status"], None) {
            "ok"
        } else {
            "warn"
        }
        .to_string(),
        detail: if command_success("gh", ["auth", "status"], None) {
            "authenticated".to_string()
        } else {
            "not authenticated; run `gh auth login`".to_string()
        },
    });

    for (label, relative_path) in required_assets() {
        let full_path = repo_dir.join(relative_path);
        checks.push(LaunchCheck {
            label: label.to_string(),
            status: if full_path.exists() { "ok" } else { "warn" }.to_string(),
            detail: full_path.display().to_string(),
        });
    }

    let social_preview = find_social_preview_asset(repo_dir);
    checks.push(LaunchCheck {
        label: "social preview".to_string(),
        status: if social_preview.is_some() {
            "ok"
        } else {
            "warn"
        }
        .to_string(),
        detail: social_preview
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| {
                "missing; create docs/social-preview.png or .jpg before launch".to_string()
            }),
    });
    checks.push(LaunchCheck {
        label: "release tag".to_string(),
        status: "ok".to_string(),
        detail: release_tag.to_string(),
    });

    checks
}

fn build_commands(plan: &LaunchPlan, wrote_artifacts: bool) -> Vec<String> {
    let base_dir = "~/.eidra/generated/launch/github";
    let repo = plan
        .repo_slug
        .clone()
        .unwrap_or_else(|| "<owner>/<repo>".to_string());
    let mut commands = Vec::new();

    if !wrote_artifacts {
        let generate_command = if let Some(repo_url) = plan.repo_url.as_ref() {
            format!(
                "eidra launch github --write --repo-dir {} --repo-url {}",
                shell_quote(&plan.repo_dir),
                shell_quote(repo_url)
            )
        } else {
            format!(
                "eidra launch github --write --repo-dir {}",
                shell_quote(&plan.repo_dir)
            )
        };
        commands.push(generate_command);
    }

    commands.push(format!("{}/apply-with-gh.sh {}", base_dir, repo));
    commands.push("Open GitHub settings and upload the social preview image.".to_string());
    commands.push("Paste DISCUSSION_ANNOUNCEMENT.md into a pinned GitHub Discussion.".to_string());
    commands.push("Send warm DMs before public X / LinkedIn posts.".to_string());
    commands
}

fn build_next_steps(checks: &[LaunchCheck], wrote_artifacts: bool) -> Vec<String> {
    let mut steps = Vec::new();

    if has_warning(checks, "git worktree") {
        steps.push("If you want git-based auto-detection, run from a cloned repo or initialize git metadata for this copy.".to_string());
    }
    if has_warning(checks, "origin url") {
        steps.push("Configure `remote.origin.url` or pass `--repo-url` so launch copy can use the real repository URL.".to_string());
    }
    if has_warning(checks, "repo slug") {
        steps.push("Pass a GitHub repo URL or run `apply-with-gh.sh <owner>/<repo>` explicitly so `gh` commands know which repository to edit.".to_string());
    }
    if has_warning(checks, "gh cli") {
        steps.push("Install GitHub CLI (`gh`) before applying launch automation.".to_string());
    }
    if has_warning(checks, "gh auth") {
        steps.push("Authenticate GitHub CLI with `gh auth login`.".to_string());
    }
    if has_warning(checks, "social preview") {
        steps.push("Run `python3 scripts/generate_social_preview.py` or create a 1280x640 GitHub social preview image before launch.".to_string());
    }
    if !wrote_artifacts {
        steps.push(
            "Run `eidra launch github --write` to generate ready-to-paste launch assets."
                .to_string(),
        );
    }
    steps.push(
        "Apply the generated GitHub metadata and release with `apply-with-gh.sh`.".to_string(),
    );
    steps.push("Send warm DMs first, then post the public launch copy.".to_string());
    steps
}

fn render_commands_markdown(plan: &LaunchPlan, script_path: &Path) -> String {
    let repo = plan.repo_slug.as_deref().unwrap_or("<owner>/<repo>");
    format!(
        "# GitHub Launch Commands\n\n1. Generate launch assets:\n\n```bash\neidra launch github --write --repo-dir {}\n```\n\n2. Apply GitHub metadata and create the release:\n\n```bash\n{} {}\n```\n\n3. Manual follow-up:\n\n- Upload the GitHub social preview image in repository settings.\n- Paste `DISCUSSION_ANNOUNCEMENT.md` into a pinned discussion.\n- Send warm DMs before public social posts.\n",
        shell_quote(&plan.repo_dir),
        script_path.display(),
        repo
    )
}

fn render_apply_script(plan: &LaunchPlan, base_dir: &Path) -> String {
    let repo = plan.repo_slug.as_deref().unwrap_or("<owner>/<repo>");
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nBASE_DIR=\"{}\"\nREPO=\"${{1:-{}}}\"\nTAG=\"${{2:-{}}}\"\n\nif ! command -v gh >/dev/null 2>&1; then\n  echo \"gh CLI is not installed. Install GitHub CLI first.\" >&2\n  exit 1\nfi\n\nif ! gh auth status >/dev/null 2>&1; then\n  echo \"gh is not authenticated. Run: gh auth login\" >&2\n  exit 1\nfi\n\nif [ ! -f \"$BASE_DIR/ABOUT_OPTION_A.txt\" ]; then\n  echo \"Launch artifacts not found in $BASE_DIR. Run: eidra launch github --write\" >&2\n  exit 1\nfi\n\ngh repo edit \"$REPO\" \\\n  --description \"$(cat \"$BASE_DIR/ABOUT_OPTION_A.txt\")\" \\\n  --homepage \"$(cat \"$BASE_DIR/WEBSITE.txt\")\" \\\n  --enable-discussions\n\nwhile IFS= read -r topic; do\n  [ -n \"$topic\" ] || continue\n  gh repo edit \"$REPO\" --add-topic \"$topic\"\ndone < \"$BASE_DIR/TOPICS.txt\"\n\ngh release create \"$TAG\" \\\n  --repo \"$REPO\" \\\n  --title \"$(cat \"$BASE_DIR/RELEASE_TITLE.txt\")\" \\\n  --notes-file \"$BASE_DIR/RELEASE_NOTES.md\"\n\necho \"GitHub metadata and release created for $REPO.\"\necho \"Next manual steps:\"\necho \"  1. Upload the social preview image in GitHub settings.\"\necho \"  2. Paste DISCUSSION_ANNOUNCEMENT.md into a pinned discussion.\"\necho \"  3. Send warm DMs before public posts.\"\n",
        base_dir.display(),
        repo,
        plan.release_tag
    )
}

fn required_assets() -> [(&'static str, &'static str); 12] {
    [
        ("README", "README.md"),
        ("For Everyone", "docs/for-everyone.md"),
        ("Use Cases", "docs/use-cases.md"),
        ("What Is Eidra", "docs/what-is-eidra.md"),
        ("For Developers", "docs/for-developers.md"),
        ("Architecture", "docs/architecture.md"),
        ("Media Kit", "docs/media-kit.md"),
        ("Outreach", "docs/outreach.md"),
        ("Launch Kit", "docs/launch/github-launch-kit.md"),
        ("Launch Checklist", "docs/launch-checklist.md"),
        ("Demo GIF", "docs/demo.gif"),
        ("AGENTS", "AGENTS.md"),
    ]
}

fn supported_targets() -> Vec<(&'static str, &'static str)> {
    vec![("github", "GitHub launch preparation and release automation")]
}

fn normalize_target(target: &str) -> String {
    match target.trim().to_ascii_lowercase().as_str() {
        "" => "github".to_string(),
        "gh" => "github".to_string(),
        other => other.to_string(),
    }
}

fn resolve_repo_dir(repo_dir: Option<String>) -> Result<PathBuf> {
    let repo_dir = match repo_dir {
        Some(value) => PathBuf::from(value),
        None => std::env::current_dir().context("failed to determine current directory")?,
    };
    if !repo_dir.exists() {
        bail!("repo directory does not exist: {}", repo_dir.display());
    }
    Ok(repo_dir.canonicalize().unwrap_or(repo_dir))
}

fn is_git_worktree(repo_dir: &Path) -> bool {
    command_success(
        "git",
        [
            "-C",
            &repo_dir.display().to_string(),
            "rev-parse",
            "--is-inside-work-tree",
        ],
        None,
    )
}

fn detect_origin_url(repo_dir: &Path) -> Option<String> {
    command_output(
        "git",
        [
            "-C",
            &repo_dir.display().to_string(),
            "config",
            "--get",
            "remote.origin.url",
        ],
        None,
    )
    .map(|value| normalize_repo_url(&value))
}

fn detect_workspace_repository(repo_dir: &Path) -> Option<String> {
    let cargo_toml = repo_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(cargo_toml).ok()?;

    content.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("repository") {
            return None;
        }

        let (_, value) = trimmed.split_once('=')?;
        let value = value.trim().trim_matches('"');
        if value.is_empty() {
            None
        } else {
            Some(normalize_repo_url(value))
        }
    })
}

fn normalize_repo_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    let normalized = if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        format!("https://github.com/{}", rest)
    } else if let Some(rest) = trimmed.strip_prefix("ssh://git@github.com/") {
        format!("https://github.com/{}", rest)
    } else {
        trimmed.to_string()
    };

    normalized.trim_end_matches(".git").to_string()
}

fn repo_slug_from_url(url: &str) -> Option<String> {
    let normalized = normalize_repo_url(url);
    normalized
        .strip_prefix("https://github.com/")
        .or_else(|| normalized.strip_prefix("http://github.com/"))
        .map(|value| value.trim_end_matches('/').to_string())
}

fn find_social_preview_asset(repo_dir: &Path) -> Option<PathBuf> {
    [
        "docs/social-preview.png",
        "docs/social-preview.jpg",
        "docs/social-preview.jpeg",
    ]
    .iter()
    .map(|relative_path| repo_dir.join(relative_path))
    .find(|path| path.exists())
}

fn command_success<const N: usize>(program: &str, args: [&str; N], dir: Option<&Path>) -> bool {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    command
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn command_output<const N: usize>(
    program: &str,
    args: [&str; N],
    dir: Option<&Path>,
) -> Option<String> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn has_warning(checks: &[LaunchCheck], label: &str) -> bool {
    checks
        .iter()
        .find(|check| check.label == label)
        .map(|check| check.status != "ok")
        .unwrap_or(false)
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || "/._:-".contains(char))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_target_maps_aliases() {
        assert_eq!(normalize_target(""), "github");
        assert_eq!(normalize_target("gh"), "github");
        assert_eq!(normalize_target("github"), "github");
    }

    #[test]
    fn normalize_repo_url_handles_git_remote_formats() {
        assert_eq!(
            normalize_repo_url("git@github.com:hanabi-jpn/eidra.git"),
            "https://github.com/hanabi-jpn/eidra"
        );
        assert_eq!(
            normalize_repo_url("ssh://git@github.com/hanabi-jpn/eidra.git"),
            "https://github.com/hanabi-jpn/eidra"
        );
        assert_eq!(
            normalize_repo_url("https://github.com/hanabi-jpn/eidra.git"),
            "https://github.com/hanabi-jpn/eidra"
        );
    }

    #[test]
    fn repo_slug_is_extracted() {
        assert_eq!(
            repo_slug_from_url("https://github.com/hanabi-jpn/eidra"),
            Some("hanabi-jpn/eidra".to_string())
        );
        assert_eq!(
            repo_slug_from_url("git@github.com:hanabi-jpn/eidra.git"),
            Some("hanabi-jpn/eidra".to_string())
        );
    }

    #[test]
    fn release_notes_include_launch_automation() {
        let notes = render_release_notes("v0.1.0");
        assert!(notes.contains("`eidra launch github`"));
        assert!(notes.contains("## Eidra v0.1.0"));
    }

    #[test]
    fn apply_script_uses_detected_repo_slug() {
        let plan = LaunchPlan {
            target: "github".to_string(),
            title: "Eidra launch automation for GitHub".to_string(),
            repo_dir: "/tmp/eidra".to_string(),
            repo_url: Some("https://github.com/hanabi-jpn/eidra".to_string()),
            repo_slug: Some("hanabi-jpn/eidra".to_string()),
            website: "https://github.com/hanabi-jpn/eidra".to_string(),
            release_tag: "v0.1.0".to_string(),
            release_title: "v0.1.0 — Local-First Trust Layer for AI Development".to_string(),
            about_options: vec![ABOUT_OPTION_A.to_string()],
            topics: TOPICS.iter().map(|topic| (*topic).to_string()).collect(),
            social_preview: SocialPreviewBrief {
                headline: "Eidra".to_string(),
                subhead: "Local-first trust layer for AI development".to_string(),
                support_line: "Proxy + MCP firewall + live dashboard".to_string(),
                visual: "demo".to_string(),
            },
            release_summary: RELEASE_SUMMARY.to_string(),
            feedback_cta: FEEDBACK_CTA.to_string(),
            builder_cta: BUILDER_CTA.to_string(),
            release_notes: render_release_notes("v0.1.0"),
            discussion_announcement: render_discussion_announcement(),
            posts: LaunchPosts {
                x: render_x_post("https://github.com/hanabi-jpn/eidra"),
                x_ja: render_x_post_ja("https://github.com/hanabi-jpn/eidra"),
                linkedin: render_linkedin_post(),
                warm_dm_en: render_warm_dm_en("https://github.com/hanabi-jpn/eidra"),
                warm_dm_ja: render_warm_dm_ja("https://github.com/hanabi-jpn/eidra"),
            },
            checks: Vec::new(),
            commands: Vec::new(),
            next_steps: Vec::new(),
            artifacts: Vec::new(),
        };

        let script = render_apply_script(&plan, Path::new("/tmp/eidra-launch"));
        assert!(script.contains("hanabi-jpn/eidra"));
        assert!(script.contains("gh release create"));
    }

    #[test]
    fn detect_workspace_repository_reads_cargo_toml() {
        let test_dir = std::env::temp_dir().join(format!(
            "eidra-launch-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&test_dir).expect("temp dir should be created");
        std::fs::write(
            test_dir.join("Cargo.toml"),
            "[workspace.package]\nrepository = \"https://github.com/hanabi-jpn/eidra.git\"\n",
        )
        .expect("cargo toml should be written");

        let repository =
            detect_workspace_repository(&test_dir).expect("repository should be detected");
        assert_eq!(repository, "https://github.com/hanabi-jpn/eidra");

        std::fs::remove_dir_all(test_dir).expect("temp dir should be removed");
    }

    #[test]
    fn shell_quote_uses_single_quotes_for_special_characters() {
        assert_eq!(shell_quote("/tmp/eidra repo"), "'/tmp/eidra repo'");
        assert_eq!(shell_quote("repo$HOME"), "'repo$HOME'");
        assert_eq!(shell_quote("repo'quoted"), "'repo'\\''quoted'");
    }
}
