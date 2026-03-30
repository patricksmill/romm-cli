use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::RommClient;
use crate::commands::print::print_roms_table;
use crate::commands::OutputFormat;
use crate::endpoints::roms::GetRoms;
use crate::services::RomService;

/// CLI entrypoint for listing/searching ROMs via `/api/roms`.
#[derive(Args, Debug)]
pub struct RomsCommand {
    #[command(subcommand)]
    pub action: Option<RomsAction>,

    /// Search term to filter roms
    #[arg(long, global = true, visible_aliases = ["query", "q"])]
    pub search_term: Option<String>,

    /// Filter by platform ID
    #[arg(long, global = true, visible_alias = "id")]
    pub platform_id: Option<u64>,

    /// Page size limit
    #[arg(long, global = true)]
    pub limit: Option<u32>,

    /// Page offset
    #[arg(long, global = true)]
    pub offset: Option<u32>,

    /// Output as JSON (overrides global --json when set).
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum RomsAction {
    /// List available ROMs (default)
    #[command(visible_alias = "ls")]
    List,
    /// Get detailed information for a single ROM
    #[command(visible_alias = "info")]
    Get {
        /// The ID of the ROM
        id: u64,
    },
}

pub async fn handle(cmd: RomsCommand, client: &RommClient, format: OutputFormat) -> Result<()> {
    let action = cmd.action.unwrap_or(RomsAction::List);

    match action {
        RomsAction::List => {
            let ep = GetRoms {
                search_term: cmd.search_term.clone(),
                platform_id: cmd.platform_id,
                collection_id: None,
                limit: cmd.limit,
                offset: cmd.offset,
            };

            let service = RomService::new(client);
            let results = service.search_roms(&ep).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                }
                OutputFormat::Text => {
                    print_roms_table(&results);
                }
            }
        }
        RomsAction::Get { id } => {
            let service = RomService::new(client);
            let rom = service.get_rom(id).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&rom)?);
                }
                OutputFormat::Text => {
                    // For now, reuse the table printer for a single item
                    // or just pretty-print the JSON.
                    println!("{}", serde_json::to_string_pretty(&rom)?);
                }
            }
        }
    }

    Ok(())
}
