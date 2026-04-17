use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::client::RommClient;
use crate::core::download::download_directory;
use crate::core::utils;
use crate::endpoints::roms::GetRoms;
use crate::services::RomService;

/// Maximum number of concurrent download connections.
const DEFAULT_CONCURRENCY: usize = 4;

/// Download a ROM to the local filesystem with a progress bar.
#[derive(Args, Debug)]
pub struct DownloadCommand {
    #[command(subcommand)]
    pub action: Option<DownloadAction>,

    /// ID of the ROM to download (legacy, use 'download one <id>' or positional)
    pub rom_id: Option<u64>,

    /// Directory to save the ROM zip(s) to
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,

    /// Download all ROMs matching the given filters concurrently (legacy, use 'download batch')
    #[arg(long, global = true)]
    pub batch: bool,

    /// Filter by platform ID
    #[arg(long, global = true)]
    pub platform_id: Option<u64>,

    /// Filter by search term
    #[arg(long, global = true)]
    pub search_term: Option<String>,

    /// Maximum concurrent downloads (default: 4)
    #[arg(long, default_value_t = DEFAULT_CONCURRENCY, global = true)]
    pub jobs: usize,

    /// Resume interrupted downloads instead of re-downloading
    #[arg(long, default_value_t = true, global = true)]
    pub resume: bool,
}

#[derive(Subcommand, Debug)]
pub enum DownloadAction {
    /// Download a single ROM by ID
    #[command(visible_alias = "one")]
    Id {
        /// ID of the ROM
        id: u64,
    },
    /// Download multiple ROMs matching filters
    #[command(visible_alias = "all")]
    Batch,
}

fn make_progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}",
    )
    .unwrap()
    .progress_chars("#>-")
}

async fn download_one(
    client: &RommClient,
    rom_id: u64,
    name: &str,
    save_path: &std::path::Path,
    pb: ProgressBar,
) -> Result<()> {
    pb.set_message(name.to_string());

    client
        .download_rom(rom_id, save_path, {
            let pb = pb.clone();
            move |received, total| {
                if pb.length() != Some(total) {
                    pb.set_length(total);
                }
                pb.set_position(received);
            }
        })
        .await?;

    pb.finish_with_message(format!("✓ {name}"));
    Ok(())
}

pub async fn handle(cmd: DownloadCommand, client: &RommClient) -> Result<()> {
    let output_dir = cmd.output.unwrap_or_else(download_directory);

    // Ensure output directory exists.
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| anyhow!("create download dir {:?}: {e}", output_dir))?;

    // Determine if we are in batch mode.
    // In order of priority: subcommand 'batch', then legacy '--batch' flag.
    let is_batch = matches!(cmd.action, Some(DownloadAction::Batch)) || cmd.batch;

    if is_batch {
        // ── Batch mode ─────────────────────────────────────────────────
        if cmd.platform_id.is_none() && cmd.search_term.is_none() {
            return Err(anyhow!(
                "Batch download requires at least --platform-id or --search-term to scope the download"
            ));
        }

        let ep = GetRoms {
            search_term: cmd.search_term.clone(),
            platform_id: cmd.platform_id,
            collection_id: None,
            smart_collection_id: None,
            virtual_collection_id: None,
            limit: Some(9999),
            offset: None,
        };

        let service = RomService::new(client);
        let results = service.search_roms(&ep).await?;

        if results.items.is_empty() {
            println!("No ROMs found matching the given filters.");
            return Ok(());
        }

        println!(
            "Found {} ROM(s). Starting download with {} concurrent connections...",
            results.items.len(),
            cmd.jobs
        );

        let mp = MultiProgress::new();
        let semaphore = Arc::new(Semaphore::new(cmd.jobs));
        let mut handles = Vec::new();

        for rom in results.items {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = client.clone();
            let dir = output_dir.clone();
            let pb = mp.add(ProgressBar::new(0));
            pb.set_style(make_progress_style());

            let name = rom.name.clone();
            let rom_id = rom.id;
            let base = utils::sanitize_filename(&rom.fs_name);
            let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(&base);
            let save_path = dir.join(format!("{stem}.zip"));

            handles.push(tokio::spawn(async move {
                let result = download_one(&client, rom_id, &name, &save_path, pb).await;
                drop(permit);
                if let Err(e) = &result {
                    eprintln!("error downloading {name} (id={rom_id}): {e}");
                }
                result
            }));
        }

        let mut successes = 0u32;
        let mut failures = 0u32;
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => successes += 1,
                _ => failures += 1,
            }
        }

        println!("\nBatch complete: {successes} succeeded, {failures} failed.");
    } else {
        // ── Single ROM mode ────────────────────────────────────────────
        let rom_id = if let Some(DownloadAction::Id { id }) = cmd.action {
            id
        } else {
            cmd.rom_id
                .ok_or_else(|| anyhow!("ROM ID is required (e.g. 'download 123' or 'download batch --search-term ...')"))?
        };

        let save_path = output_dir.join(format!("rom_{rom_id}.zip"));

        let mp = MultiProgress::new();
        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(make_progress_style());

        download_one(client, rom_id, &format!("ROM {rom_id}"), &save_path, pb).await?;

        println!("Saved to {:?}", save_path);
    }

    Ok(())
}
