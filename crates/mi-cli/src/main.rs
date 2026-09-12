mod cli;
mod commands;
mod settings;

use clap::Parser;
use cli::{Cli, Command};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose, cli.quiet);

    let app_config = settings::load_config(cli.config.as_deref())?;
    let pool = mi_storage::init_db(&app_config.db_path).await?;

    match cli.command {
        Command::Scope { action } => commands::scope::handle(&pool, action).await?,
        Command::Config { action } => commands::config_cmd::handle(&app_config, action).await?,
        Command::Status => commands::status::handle(&pool, &app_config).await?,
        Command::Host { ip } => commands::host::handle(&pool, &app_config, ip).await?,
    }

    Ok(())
}

fn init_logging(verbose: u8, quiet: bool) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = if quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level))
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
