use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::RommClient;
use crate::commands::library_scan::{
    run_scan_library_flow, ScanCacheInvalidate, ScanLibraryOptions,
};
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
    /// Upload a ROM file to a platform
    #[command(visible_alias = "up")]
    Upload {
        /// The platform ID to upload to
        platform_id: u64,
        /// The file to upload
        file: PathBuf,
        /// Trigger a library scan after upload completes
        #[arg(short, long)]
        scan: bool,
        /// Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)
        #[arg(long, requires = "scan")]
        wait: bool,
        /// Max seconds to wait when `--wait` is set (default: 3600)
        #[arg(long, requires = "wait")]
        wait_timeout_secs: Option<u64>,
    },
}

fn make_progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}",
    )
    .unwrap()
    .progress_chars("#>-")
}

async fn upload_one(
    client: &RommClient,
    platform_id: u64,
    file_path: std::path::PathBuf,
    pb: ProgressBar,
) -> Result<()> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    pb.set_message(format!("Uploading {}", filename));

    client
        .upload_rom(platform_id, &file_path, {
            let pb = pb.clone();
            move |uploaded, total| {
                if pb.length() != Some(total) {
                    pb.set_length(total);
                }
                pb.set_position(uploaded);
            }
        })
        .await?;

    pb.finish_with_message(format!("✓ Upload complete: {}", filename));
    Ok(())
}

pub async fn handle(cmd: RomsCommand, client: &RommClient, format: OutputFormat) -> Result<()> {
    let action = cmd.action.unwrap_or(RomsAction::List);

    match action {
        RomsAction::List => {
            let ep = GetRoms {
                search_term: cmd.search_term.clone(),
                platform_id: cmd.platform_id,
                collection_id: None,
                smart_collection_id: None,
                virtual_collection_id: None,
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
        RomsAction::Upload {
            file,
            platform_id,
            scan,
            wait,
            wait_timeout_secs,
        } => {
            if !file.exists() {
                anyhow::bail!("File or directory does not exist: {:?}", file);
            }

            let mut files = Vec::new();
            if file.is_dir() {
                let mut entries = tokio::fs::read_dir(&file).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_file() {
                        files.push(path);
                    }
                }
                files.sort(); // Consistent order
            } else {
                files.push(file);
            }

            if files.is_empty() {
                println!("No files found to upload.");
                return Ok(());
            }

            if files.len() > 1 {
                println!("Found {} files to upload.", files.len());
            }

            let mp = indicatif::MultiProgress::new();
            let mut successes = 0u32;
            for path in files {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(make_progress_style());
                match upload_one(client, platform_id, path.clone(), pb).await {
                    Ok(()) => successes += 1,
                    Err(e) => eprintln!("Error uploading {:?}: {}", path, e),
                }
            }

            if scan {
                if successes == 0 {
                    eprintln!("Skipping library scan: no uploads completed successfully.");
                } else {
                    let options = ScanLibraryOptions {
                        wait,
                        wait_timeout: Duration::from_secs(wait_timeout_secs.unwrap_or(3600)),
                        cache_invalidate: if wait {
                            ScanCacheInvalidate::Platform(platform_id)
                        } else {
                            ScanCacheInvalidate::None
                        },
                    };
                    run_scan_library_flow(client, options, format).await?;
                }
            }
        }
    }

    Ok(())
}
