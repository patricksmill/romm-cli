use anyhow::Result;
use clap::Args;

use crate::client::RommClient;
use crate::commands::print::print_roms_table;
use crate::commands::OutputFormat;
use crate::endpoints::roms::GetRoms;
use crate::services::RomService;

/// CLI entrypoint for listing/searching ROMs via `/api/roms`.
#[derive(Args, Debug)]
pub struct RomsCommand {
    /// Search term to filter roms
    #[arg(long)]
    pub search_term: Option<String>,

    /// Filter by platform ID
    #[arg(long)]
    pub platform_id: Option<u64>,

    /// Page size limit
    #[arg(long)]
    pub limit: Option<u32>,

    /// Page offset
    #[arg(long)]
    pub offset: Option<u32>,

    /// Output as JSON (overrides global --json when set).
    #[arg(long)]
    pub json: bool,
}

pub async fn handle(
    cmd: RomsCommand,
    client: &RommClient,
    format: OutputFormat,
) -> Result<()> {
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

    Ok(())
}
