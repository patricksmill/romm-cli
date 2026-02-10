use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::RommClient;
use crate::endpoints::platforms::ListPlatforms;

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
    let platforms = client.call(&ListPlatforms::default()).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&platforms)?);
        return Ok(());
    }

    for p in platforms {
        let display_name = p
            .display_name
            .as_ref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&p.name);

        println!(
            "{}\t{}\t{}\troms:{}\tfirmware:{}",
            p.id,
            p.slug,
            display_name,
            p.rom_count,
            p.firmware.len()
        );
    }

    Ok(())
}
