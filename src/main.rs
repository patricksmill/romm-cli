//! Binary entrypoint for `romm-cli`.

use anyhow::Result;
use clap::Parser;
use romm_cli::commands::{run, Cli};
use romm_cli::config::load_config;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    let config = load_config()?;

    run(cli, config).await
}
