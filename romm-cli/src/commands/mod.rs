//! Top-level CLI command handling.
//!
//! The `Cli` type (derived from `clap`) describes the public command-line
//! interface. Each subcommand lives in its own module and is free to use
//! `RommClient` directly. The TUI is just another subcommand.

use clap::{Parser, Subcommand};

use romm_api::client::RommClient;
use romm_api::config::Config;
use romm_api::error::RommError;

pub mod api;
pub mod auth;
pub mod cache;
pub mod completions;
pub mod download;
pub mod init;
pub mod library_scan;
pub mod platforms;
pub mod print;
pub mod roms;
pub mod scan;
pub mod sync;
pub mod update;

/// Defines how a command should format its output for the user.
#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    /// Human-readable text format, often with tables and aligned columns.
    Text,
    /// Machine-friendly JSON format, useful for scripting and integration.
    Json,
}

impl OutputFormat {
    /// Resolves the effective output format based on global and command-specific flags.
    ///
    /// If either the global `--json` flag or a local `--json` flag is set,
    /// this returns [`OutputFormat::Json`].
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
/// This structure defines the global flags and available subcommands
/// for the `romm-cli` binary.
#[derive(Parser, Debug)]
#[command(
    name = "romm-cli",
    version,
    about = "Rust CLI and TUI for the ROMM API",
    infer_subcommands = true,
    arg_required_else_help = true,
    after_help = "Exit codes: 0 success, 1 general failure, 2 usage, 3 config/auth, 4 API/network.\n\
                  See README \"Exit codes\" for scripting examples.\n\
                  JSON output shapes: docs/json-output.md"
)]
pub struct Cli {
    /// Increase output verbosity (logs requests to stderr).
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output JSON instead of human-readable text where supported.
    #[arg(long, global = true)]
    pub json: bool,

    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// All top-level commands supported by the CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create or update user configuration.
    #[command(visible_alias = "setup")]
    Init(init::InitCommand),
    /// Low-level access to any RomM API endpoint.
    #[command(visible_alias = "call")]
    Api(api::ApiCommand),
    /// Manage gaming platforms.
    #[command(visible_aliases = ["platform", "p", "plats"])]
    Platforms(platforms::PlatformsCommand),
    /// Manage ROM files and metadata.
    #[command(visible_aliases = ["rom", "r"])]
    Roms(Box<roms::RomsCommand>),
    /// Trigger a library scan on the RomM server.
    Scan(scan::ScanCommand),
    /// Save-sync workflows (device registration, planning, and execution).
    Sync(sync::SyncCommand),
    /// Download a ROM or related extras from the server.
    #[command(visible_aliases = ["dl", "get"])]
    Download(download::DownloadCommand),
    /// Manage the local persistent cache.
    Cache(cache::CacheCommand),
    /// Manage authentication credentials.
    Auth(auth::AuthCommand),
    /// Check for and install application updates.
    Update,
    /// Generate shell completion scripts.
    Completions(completions::CompletionsCommand),
}

/// Main entrypoint for running a CLI command.
///
/// This function initializes the [`RommClient`] and dispatches the
/// chosen command to its respective handler.
pub async fn run(cli: Cli, config: Config) -> Result<(), RommError> {
    let client = RommClient::new(&config, cli.verbose)?;

    match cli.command {
        Commands::Completions(_) => Err(RommError::Other(
            "internal error: completions must be handled in main".into(),
        )),
        Commands::Init(_) => Err(RommError::Other(
            "internal error: init must be handled before load_config".into(),
        )),
        command => crate::frontend::cli::run(command, &client, cli.json, cli.verbose).await,
    }
}
