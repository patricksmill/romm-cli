use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::RommClient;
use crate::commands::print::print_platforms_table;
use crate::commands::OutputFormat;
use crate::services::PlatformService;

/// CLI entrypoint for platform-related operations.
#[derive(Args, Debug)]
pub struct PlatformsCommand {
    #[command(subcommand)]
    pub action: Option<PlatformsAction>,

    /// Output as JSON (overrides global --json when set).
    #[arg(long, global = true)]
    pub json: bool,
}

/// Specific action to perform for `romm-cli platforms`.
#[derive(Subcommand, Debug)]
pub enum PlatformsAction {
    /// List all platforms (default)
    #[command(visible_alias = "ls")]
    List,
    /// Get details for a specific platform
    #[command(visible_alias = "info")]
    Get {
        /// The ID of the platform
        id: u64,
    },
}

pub async fn handle(
    cmd: PlatformsCommand,
    client: &RommClient,
    format: OutputFormat,
) -> Result<()> {
    let action = cmd.action.unwrap_or(PlatformsAction::List);

    let service = PlatformService::new(client);
    match action {
        PlatformsAction::List => {
            let platforms = service.list_platforms().await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&platforms)?);
                }
                OutputFormat::Text => {
                    print_platforms_table(&platforms);
                }
            }
        }
        PlatformsAction::Get { id } => {
            let platform = service.get_platform(id).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&platform)?);
                }
                OutputFormat::Text => {
                    println!("{}", serde_json::to_string_pretty(&platform)?);
                }
            }
        }
    }

    Ok(())
}
