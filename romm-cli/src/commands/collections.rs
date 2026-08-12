//! `collections` — list, inspect, and delete RomM collections.

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use dialoguer::Confirm;

use crate::cli_presentation::CliPresentation;
use crate::commands::OutputFormat;
use romm_api::client::RommClient;
use romm_api::core::collections::{
    delete_collection, get_collection, list_collections, CollectionKind,
};

#[derive(Args, Debug)]
#[command(
    about = "List and manage manual, smart, and virtual collections",
    after_help = "Examples:\n  \
      romm-cli collections list --type all --json\n  \
      romm-cli collections get 3 --type manual\n  \
      romm-cli collections delete 2 --type smart --yes"
)]
pub struct CollectionsCommand {
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub action: CollectionsAction,
}

#[derive(Subcommand, Debug)]
pub enum CollectionsAction {
    /// List collections.
    List {
        #[arg(long, default_value = "all")]
        r#type: String,
    },
    /// Get one collection by id.
    Get {
        id: String,
        #[arg(long, default_value = "manual")]
        r#type: String,
    },
    /// Delete a manual or smart collection.
    Delete {
        id: u64,
        #[arg(long, default_value = "manual")]
        r#type: String,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn handle(
    cmd: CollectionsCommand,
    client: &RommClient,
    presentation: CliPresentation,
) -> Result<()> {
    let format = presentation.format;
    match cmd.action {
        CollectionsAction::List { r#type } => {
            let kind = CollectionKind::parse(&r#type).map_err(|e| anyhow!("{e}"))?;
            let rows = list_collections(client, kind).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
                OutputFormat::Text => {
                    for c in &rows {
                        let tag = if c.is_virtual {
                            "virtual"
                        } else if c.is_smart {
                            "smart"
                        } else {
                            "manual"
                        };
                        println!(
                            "[{tag}] {}  {}  roms={}",
                            c.id,
                            c.name,
                            c.rom_count.unwrap_or(0)
                        );
                    }
                }
            }
        }
        CollectionsAction::Get { id, r#type } => {
            let kind = CollectionKind::parse(&r#type).map_err(|e| anyhow!("{e}"))?;
            if kind == CollectionKind::All {
                return Err(anyhow!("specify --type manual, smart, or virtual for get"));
            }
            let value = get_collection(client, kind, &id).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&value)?),
                OutputFormat::Text => println!("{}", serde_json::to_string_pretty(&value)?),
            }
        }
        CollectionsAction::Delete { id, r#type, yes } => {
            let kind = CollectionKind::parse(&r#type).map_err(|e| anyhow!("{e}"))?;
            if kind == CollectionKind::All || kind == CollectionKind::Virtual {
                return Err(anyhow!("only manual and smart collections can be deleted"));
            }
            if !yes {
                let ok = Confirm::new()
                    .with_prompt(format!("Delete {} collection {id}?", r#type))
                    .default(false)
                    .interact()?;
                if !ok {
                    return Ok(());
                }
            }
            let value = delete_collection(client, kind, id).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&value)?),
                OutputFormat::Text => println!("Deleted collection {id}"),
            }
        }
    }
    Ok(())
}
