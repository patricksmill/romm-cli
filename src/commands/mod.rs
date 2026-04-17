//! Top-level CLI command handling.
//!
//! The `Cli` type (derived from `clap`) describes the public command-line
//! interface. Each subcommand lives in its own module and is free to use
//! `RommClient` directly. The TUI is just another subcommand.

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::client::RommClient;
use crate::config::Config;

pub mod api;
pub mod cache;
pub mod download;
pub mod init;
pub mod platforms;
pub mod print;
pub mod roms;
pub mod update;

/// How a command should format its output.
#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    /// Human-readable text (tables, aligned columns, etc.).
    Text,
    /// Machine-friendly JSON (pretty-printed by default).
    Json,
}

impl OutputFormat {
    /// Resolve the effective output format from global and per-command flags.
    pub fn from_flags(global_json: bool, local_json: bool) -> Self {
        if global_json || local_json {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        }
    }
}

/// Top-level CLI entrypoint for `romm-cli`.
///
/// This binary can be used both as:
/// - a **TUI launcher** (`romm-cli tui`), and
/// - a **scripting-friendly CLI** for platforms/ROMs/API calls.
#[derive(Parser, Debug)]
#[command(
    name = "romm-cli",
    version,
    about = "Rust CLI and TUI for the ROMM API",
    infer_subcommands = true,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Increase output verbosity
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output JSON instead of human-readable text where supported.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// All top-level commands supported by the CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create or update user config (~/.config/romm-cli/config.json or %APPDATA%\\romm-cli\\config.json)
    #[command(visible_alias = "setup")]
    Init(init::InitCommand),
    /// Launch interactive TUI for exploring API endpoints
    #[cfg(feature = "tui")]
    Tui,
    /// Launch interactive TUI (stub for disabled feature)
    #[cfg(not(feature = "tui"))]
    Tui,
    /// Low-level access to any ROMM API endpoint
    #[command(visible_alias = "call")]
    Api(api::ApiCommand),
    /// Platform-related commands
    #[command(visible_aliases = ["platform", "p", "plats"])]
    Platforms(platforms::PlatformsCommand),
    /// ROM-related commands
    #[command(visible_aliases = ["rom", "r"])]
    Roms(roms::RomsCommand),
    /// Download a ROM
    #[command(visible_aliases = ["dl", "get"])]
    Download(download::DownloadCommand),
    /// Manage the persistent ROM cache
    Cache(cache::CacheCommand),
    /// Check for and install updates for romm-cli
    Update,
}

pub async fn run(cli: Cli, config: Config) -> Result<()> {
    let client = RommClient::new(&config, cli.verbose)?;

    match cli.command {
        Commands::Init(_) => {
            anyhow::bail!("internal error: init must be handled before load_config");
        }
        #[cfg(feature = "tui")]
        Commands::Tui => {
            anyhow::bail!("internal error: TUI must be started via run_interactive from main");
        }
        #[cfg(not(feature = "tui"))]
        Commands::Tui => anyhow::bail!("this feature requires the tui"),
        command => crate::frontend::cli::run(command, &client, cli.json).await?,
    }

    Ok(())
}
