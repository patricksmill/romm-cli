use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::RommClient;

#[derive(Args, Debug)]
pub struct PlatformsCommand {
    #[command(subcommand)]
    pub action: Option<PlatformsAction>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum PlatformsAction {
    /// List all platforms (default)
    List,
}

pub async fn handle(cmd: PlatformsCommand, client: &RommClient, json: bool) -> Result<()> {
    let action = cmd.action.unwrap_or(PlatformsAction::List);

    match action {
        PlatformsAction::List => list_platforms(client, cmd.json || json).await?,
    }

    Ok(())
}

async fn list_platforms(client: &RommClient, json: bool) -> Result<()> {
    let platforms = client.get_platforms().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&platforms)?);
        return Ok(());
    }

    for p in platforms {
        println!(
            "{}\t{}\t{}\troms:{}\tfirmware:{}",
            p.id,
            p.slug,
            p.display_name.as_deref().unwrap_or(&p.name),
            p.rom_count,
            p.firmware.len()
        );
    }

    Ok(())
}

