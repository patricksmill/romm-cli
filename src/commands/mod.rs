use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::client::RommClient;
use crate::config::Config;

pub mod platforms;

#[derive(Parser, Debug)]
#[command(name = "romm-cli", version, about = "Rust CLI for ROMM API")]
pub struct Cli {
    /// Increase output verbosity
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Platform-related commands
    Platforms(platforms::PlatformsCommand),
}

pub async fn run(cli: Cli, config: Config) -> Result<()> {
    let client = RommClient::new(&config)?;

    match cli.command {
        Commands::Platforms(cmd) => platforms::handle(cmd, &client, cli.verbose).await?,
    }

    Ok(())
}

