mod config;
mod types;
mod client;
mod commands;
mod endpoints;
mod tui;

use anyhow::Result;
use clap::Parser;
use config::load_config;
use commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env in development (no-op if file missing)
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    let config = load_config()?;

    commands::run(cli, config).await
}

