use anyhow::Result;
use clap::{Parser, Subcommand};
use ps_daemon::{cli as daemon_cli, runner};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "prompt-saviour", about = "Local prompt draft recovery for coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run background capture daemon (Mac POC)
    Run,
    /// List saved drafts
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Recover a draft to the system clipboard
    Recover {
        /// Draft id from `list`; latest if omitted
        id: Option<i64>,
    },
    /// Check / request macOS permissions
    Doctor,
    /// Show database path and config
    Status,
    /// Queue a draft for the running daemon (E2E / debugging)
    Inject {
        /// Prompt text to save
        text: String,
        #[arg(long, default_value = "TextEdit")]
        app: String,
        #[arg(long, default_value = "com.apple.TextEdit")]
        bundle: String,
    },
    /// Run self-contained pipeline smoke test (no OS permissions needed)
    Smoke {
        #[arg(long, default_value = "Smoke test prompt for prompt-saviour pipeline")]
        text: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("prompt_saviour=info".parse()?))
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Run => runner::run_daemon(),
        Commands::List { limit } => daemon_cli::list_drafts(limit),
        Commands::Recover { id } => daemon_cli::recover_draft(id),
        Commands::Doctor => daemon_cli::doctor(),
        Commands::Status => daemon_cli::status(),
        Commands::Inject { text, app, bundle } => daemon_cli::inject_draft(&text, &app, &bundle),
        Commands::Smoke { text } => daemon_cli::smoke_test(&text),
    }
}
