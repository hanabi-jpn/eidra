mod commands;
mod runtime;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "eidra",
    about = "Edge-native trust for humans, agents, and machines.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Eidra (generate CA, create config)
    Init,
    /// Start the intercept proxy
    Start {
        /// Listen address
        #[arg(short, long)]
        listen: Option<String>,
        /// Launch with TUI dashboard
        #[arg(short, long)]
        dashboard: bool,
    },
    /// Stop the proxy
    Stop,
    /// Scan a file or stdin for secrets
    Scan {
        /// File path to scan (reads stdin if omitted)
        path: Option<String>,
        /// Emit findings as JSON
        #[arg(long)]
        json: bool,
    },
    /// Print setup guidance for common environments
    Setup {
        /// Target environment (shell, cursor, claude-code, openai-sdk, anthropic-sdk, github-actions, mcp, list)
        target: Option<String>,
        /// Write generated setup artifacts under ~/.eidra/generated/<target>
        #[arg(long)]
        write: bool,
        /// Emit setup guidance as JSON
        #[arg(long)]
        json: bool,
    },
    /// Generate launch automation for GitHub
    Launch {
        /// Launch target (github, list)
        target: Option<String>,
        /// Write generated launch assets under ~/.eidra/generated/launch/<target>
        #[arg(long)]
        write: bool,
        /// Emit launch output as JSON
        #[arg(long)]
        json: bool,
        /// Override the GitHub repository URL
        #[arg(long)]
        repo_url: Option<String>,
        /// Override the repository directory used for readiness checks
        #[arg(long)]
        repo_dir: Option<String>,
        /// Override the release tag (defaults to v<CARGO_PKG_VERSION>)
        #[arg(long)]
        tag: Option<String>,
    },
    /// Open the TUI dashboard (starts proxy + dashboard)
    Dashboard {
        /// Listen address for proxy
        #[arg(short, long)]
        listen: Option<String>,
    },
    /// Check runtime readiness and effective configuration
    Doctor {
        /// Emit readiness as JSON
        #[arg(long)]
        json: bool,
    },
    /// Run the MCP firewall gateway
    Gateway {
        /// Listen address
        #[arg(short, long)]
        listen: Option<String>,
    },
    /// Open a zero-trace encrypted room
    Escape,
    /// Join a zero-trace encrypted room
    Join {
        /// Room ID
        room_id: String,
        /// Port to connect to
        port: u16,
    },
    /// Manage configuration
    Config {
        /// Action: show, path, edit, edit-policy, reset, validate
        action: Option<String>,
        /// Emit config output as JSON when supported
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Only init tracing for non-TUI commands (TUI takes over the terminal)
    let is_tui = matches!(
        cli.command,
        Commands::Dashboard { .. }
            | Commands::Start {
                dashboard: true,
                ..
            }
    );
    if !is_tui {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();
    }

    match cli.command {
        Commands::Start { listen, dashboard } => {
            commands::start::run(listen.as_deref(), dashboard).await?
        }
        Commands::Dashboard { listen } => commands::start::run(listen.as_deref(), true).await?,
        Commands::Doctor { json } => commands::doctor::run(json).await?,
        Commands::Gateway { listen } => commands::gateway::run(listen.as_deref()).await?,
        Commands::Scan { path, json } => commands::scan::run(path, json).await?,
        Commands::Setup {
            target,
            write,
            json,
        } => commands::setup::run(target, write, json).await?,
        Commands::Launch {
            target,
            write,
            json,
            repo_url,
            repo_dir,
            tag,
        } => commands::launch::run(target, write, json, repo_url, repo_dir, tag).await?,
        Commands::Init => commands::init::run().await?,
        Commands::Stop => commands::stop::run().await?,
        Commands::Escape => commands::escape::run().await?,
        Commands::Join { room_id, port } => commands::join::run(&room_id, port).await?,
        Commands::Config { action, json } => commands::config::run(action, json).await?,
    }

    Ok(())
}
