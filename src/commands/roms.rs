use anyhow::Result;
use clap::Args;

use crate::client::RommClient;

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
    let results = client
        .get_roms(
            cmd.search_term.as_deref(),
            cmd.platform_id,
            cmd.limit,
            cmd.offset,
        )
        .await?;

    if cmd.json || json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for r in results.items {
            println!("{}\t{}\t{}", r.id, r.platform_id, r.name);
        }
    }

    Ok(())
}

