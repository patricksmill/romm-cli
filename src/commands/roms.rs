use anyhow::Result;
use clap::Args;

use crate::client::RommClient;
use crate::endpoints::roms::GetRoms;

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
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn handle(cmd: RomsCommand, client: &RommClient, json: bool) -> Result<()> {
    let ep = GetRoms {
        search_term: cmd.search_term.clone(),
        platform_id: cmd.platform_id,
        collection_id: None,
        limit: cmd.limit,
        offset: cmd.offset,
    };

    let results = client.call(&ep).await?;

    if cmd.json || json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for r in results.items {
            println!("{}\t{}\t{}", r.id, r.platform_id, r.name);
        }
    }

    Ok(())
}
