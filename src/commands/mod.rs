use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::client::RommClient;
use crate::config::Config;

pub mod api;
pub mod platforms;
pub mod roms;

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
    /// Launch interactive TUI for exploring API endpoints
    Tui,
    /// Low-level access to any ROMM API endpoint
    Api(api::ApiCommand),
    /// Platform-related commands
    Platforms(platforms::PlatformsCommand),
    /// ROM-related commands
    Roms(roms::RomsCommand),
}

pub async fn run(cli: Cli, config: Config) -> Result<()> {
    let client = RommClient::new(&config)?;

    match cli.command {
        Commands::Tui => crate::tui::run(client).await?,
        Commands::Api(cmd) => api::handle(cmd, &client).await?,
        Commands::Platforms(cmd) => platforms::handle(cmd, &client, cli.verbose).await?,
        Commands::Roms(cmd) => roms::handle(cmd, &client, cli.verbose).await?,
    }

    Ok(())
}

