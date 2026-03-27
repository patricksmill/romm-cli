use anyhow::Result;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::client::RommClient;

/// Download a ROM to the local filesystem with a progress bar.
#[derive(Args, Debug)]
pub struct DownloadCommand {
    /// ID of the ROM to download
    pub rom_id: u64,

    /// Directory or full file path to save the ROM zip to
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub async fn handle(cmd: DownloadCommand, client: &RommClient) -> Result<()> {
    let output_path = if let Some(out) = cmd.output {
        if out.is_dir() {
            out.join(format!("rom_{}.zip", cmd.rom_id)) // fallback, actual name usually from header
        } else {
            out
        }
    } else {
        std::env::current_dir()?.join(format!("rom_{}.zip", cmd.rom_id))
    };

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    pb.set_message("Downloading...");

    client
        .download_rom(cmd.rom_id, &output_path, {
            let pb = pb.clone();
            move |received, total| {
                if pb.length() != Some(total) {
                    pb.set_length(total);
                }
                pb.set_position(received);
            }
        })
        .await?;

    pb.finish_with_message("Done!");
    tracing::info!("Saved ROM to {:?}", output_path);

    Ok(())
}
