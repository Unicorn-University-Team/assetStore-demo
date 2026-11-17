//! Demo data for assetStore (docs/spec.md). Talks to the site only through its public API.

mod api;
mod config;
mod fixtures;
mod status;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Demo data for assetStore: seed, wipe and status")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Print what would change, change nothing.
    #[arg(long, global = true)]
    dry_run: bool,
    /// Required for seed and wipe against a server that is not local.
    #[arg(long, global = true)]
    yes: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Create the demo data, or add what is missing.
    Seed,
    /// Delete the demo data, and only the demo data.
    Wipe,
    /// Count the demo users and their assets.
    Status,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::from_env()?;
    let changes_data = matches!(cli.command, Command::Seed | Command::Wipe);
    if changes_data && !config.is_local() && !cli.yes && !cli.dry_run {
        bail!(
            "{} is not a local server: add --yes to change its data",
            config.api_url
        );
    }

    match cli.command {
        Command::Status => status::run(&config).await,
        Command::Seed => bail!("seed is not implemented yet (docs/plan.md, steps 2-4)"),
        Command::Wipe => bail!("wipe is not implemented yet (docs/plan.md, step 5)"),
    }
}
