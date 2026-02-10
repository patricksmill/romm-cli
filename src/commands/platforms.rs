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
    #[arg(long)]
    pub json: bool,
}

/// Specific action to perform for `romm-cli platforms`.
#[derive(Subcommand, Debug)]
pub enum PlatformsAction {
    /// List all platforms (default)
    List,
}

pub async fn handle(
    cmd: PlatformsCommand,
    client: &RommClient,
    format: OutputFormat,
) -> Result<()> {
    let action = cmd.action.unwrap_or(PlatformsAction::List);

    match action {
        PlatformsAction::List => list_platforms(client, format).await?,
    }

    Ok(())
}

async fn list_platforms(client: &RommClient, format: OutputFormat) -> Result<()> {
    let service = PlatformService::new(client);
    let platforms = service.list_platforms().await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&platforms)?);
        }
        OutputFormat::Text => {
            print_platforms_table(&platforms);
        }
    }

    Ok(())
}
