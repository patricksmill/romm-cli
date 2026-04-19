//! Top-level `scan` command: trigger RomM `scan_library` without uploading.

use std::time::Duration;

use anyhow::Result;
use clap::Args;

use crate::client::RommClient;

use super::library_scan::{run_scan_library_flow, ScanLibraryOptions};
use super::OutputFormat;

#[derive(Args, Debug)]
pub struct ScanCommand {
    /// Wait until the scan task completes (polls every 2 seconds)
    #[arg(long)]
    pub wait: bool,

    /// Max seconds to wait when `--wait` is set (default: 3600)
    #[arg(long, requires = "wait")]
    pub wait_timeout_secs: Option<u64>,
}

pub async fn handle(cmd: ScanCommand, client: &RommClient, format: OutputFormat) -> Result<()> {
    let options = ScanLibraryOptions {
        wait: cmd.wait,
        wait_timeout: Duration::from_secs(cmd.wait_timeout_secs.unwrap_or(3600)),
    };
    run_scan_library_flow(client, options, format).await
}
