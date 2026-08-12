//! `saves` — list, download, and upload game saves.

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};

use crate::cli_presentation::CliPresentation;
use crate::commands::OutputFormat;
use romm_api::client::{RommClient, SaveUploadOptions};
use romm_api::config::{load_config, resolved_save_dir};
use romm_api::core::saves::{
    download_save_to_path, get_save, list_saves, upload_save_for_rom, SaveListFilter,
};

#[derive(Args, Debug)]
#[command(
    about = "List, download, and upload game saves",
    after_help = "Examples:\n  \
      romm-cli saves list --rom-id 42 --json\n  \
      romm-cli saves download 9 --output ./game.sav\n  \
      romm-cli saves upload --rom-id 42 ./save.srm --emulator retroarch"
)]
pub struct SavesCommand {
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub action: SavesAction,
}

#[derive(Subcommand, Debug)]
pub enum SavesAction {
    /// List saves (`GET /api/saves`).
    List {
        #[arg(long)]
        rom_id: Option<u64>,
        #[arg(long)]
        device_id: Option<String>,
        #[arg(long)]
        slot: Option<String>,
    },
    /// Get one save record.
    Get {
        id: u64,
        #[arg(long)]
        device_id: Option<String>,
    },
    /// Download save file content.
    Download {
        id: u64,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        device_id: Option<String>,
        #[arg(long)]
        session_id: Option<u64>,
    },
    /// Upload a save file for a ROM.
    Upload {
        #[arg(long)]
        rom_id: u64,
        file: PathBuf,
        #[arg(long)]
        emulator: Option<String>,
        #[arg(long)]
        slot: Option<String>,
        #[arg(long)]
        device_id: Option<String>,
        #[arg(long)]
        overwrite: bool,
    },
}

pub async fn handle(
    cmd: SavesCommand,
    client: &RommClient,
    presentation: CliPresentation,
) -> Result<()> {
    let format = presentation.format;
    match cmd.action {
        SavesAction::List {
            rom_id,
            device_id,
            slot,
        } => {
            let rows = list_saves(
                client,
                SaveListFilter {
                    rom_id,
                    device_id,
                    slot,
                },
            )
            .await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
                OutputFormat::Text => {
                    if rows.is_empty() {
                        println!("No saves.");
                    } else {
                        for s in &rows {
                            println!(
                                "{}  {}  {}",
                                s.id,
                                s.file_name,
                                s.updated_at.as_deref().unwrap_or("")
                            );
                        }
                    }
                }
            }
        }
        SavesAction::Get { id, device_id } => {
            let save = get_save(client, id, device_id.as_deref()).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&save)?),
                OutputFormat::Text => println!("{}", serde_json::to_string_pretty(&save)?),
            }
        }
        SavesAction::Download {
            id,
            output,
            device_id,
            session_id,
        } => {
            let dest = match output {
                Some(p) => p,
                None => {
                    let cfg = load_config().map_err(|e| anyhow!("{e}"))?;
                    let base = resolved_save_dir(&cfg);
                    base.join(format!("save-{id}.sav"))
                }
            };
            let path =
                download_save_to_path(client, id, &dest, device_id.as_deref(), session_id).await?;
            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "path": path.display().to_string(),
                            "save_id": id,
                        }))?
                    );
                }
                OutputFormat::Text => println!("Saved to {}", path.display()),
            }
        }
        SavesAction::Upload {
            rom_id,
            file,
            emulator,
            slot,
            device_id,
            overwrite,
        } => {
            if !file.is_file() {
                return Err(anyhow!("not a file: {}", file.display()));
            }
            let options = SaveUploadOptions {
                emulator: emulator.as_deref(),
                slot: slot.as_deref(),
                device_id: device_id.as_deref(),
                overwrite,
                ..Default::default()
            };
            let value = upload_save_for_rom(client, rom_id, &file, options).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&value)?),
                OutputFormat::Text => println!("Upload complete for rom_id={rom_id}"),
            }
        }
    }
    Ok(())
}
