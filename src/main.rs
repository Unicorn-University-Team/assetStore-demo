//! Demo data for assetStore (docs/spec.md). Talks to the site only through its public API; what the API cannot do
//! is written to a SQL file for the server (--sql-out).

mod api;
mod config;
mod files;
mod fixtures;
mod seed;
mod sql;
mod status;
mod summary;
mod users;
mod wipe;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
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
    /// Where to write the SQL for what the API cannot do (roles, dates, deleting accounts).
    #[arg(long, global = true)]
    sql_out: Option<PathBuf>,
    /// Append a Markdown summary to this file (in GitHub Actions: $GITHUB_STEP_SUMMARY).
    #[arg(long, global = true)]
    summary: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Create the demo data, or add what is missing.
    Seed,
    /// Delete the demo data, and only the demo data.
    Wipe,
    /// Count the demo users and their assets.
    Status,
    /// Status, plus the showcase assets must answer on the site (detail and preview); fails when something is broken.
    Check,
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

    let started = std::time::Instant::now();
    let mut sql = sql::SqlFile::default();
    let mut summary = match cli.command {
        Command::Status => status::run(&config, false).await?,
        Command::Check => status::run(&config, true).await?,
        Command::Seed => seed::run(&config, cli.dry_run, &mut sql).await?,
        Command::Wipe => wipe::run(&config, cli.dry_run, &mut sql).await?,
    };
    summary.seconds = started.elapsed().as_secs_f32();
    summary.print();

    if !sql.is_empty() {
        match &cli.sql_out {
            Some(path) => {
                std::fs::write(path, sql.render())
                    .with_context(|| format!("writing {}", path.display()))?;
                println!(
                    "  SQL ({} statements): {}",
                    sql.statement_count(),
                    path.display()
                );
            }
            None => println!(
                "  {} SQL statements not written: add --sql-out <file>",
                sql.statement_count()
            ),
        }
    }
    if let Some(path) = &cli.summary {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", summary.markdown(&config))?;
    }
    if !summary.problems.is_empty() {
        bail!("{} problem(s), see above", summary.problems.len());
    }
    Ok(())
}
