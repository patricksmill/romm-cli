//! Top-level `scan` command: trigger RomM `scan_library` without uploading.

use std::time::Duration;

use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::client::RommClient;
use crate::core::interrupt::InterruptContext;
use crate::services::{self, PlatformService};

use super::library_scan::{run_scan_library_flow, ScanCacheInvalidate, ScanLibraryOptions};
use super::OutputFormat;

#[derive(Args, Debug)]
pub struct ScanCommand {
    /// Restrict scan to one or more platform slugs (comma-separated); passed as `platform_slugs` task kwargs
    #[arg(long)]
    pub platform: Option<String>,

    /// Wait until the scan task completes (polls every 2 seconds)
    #[arg(long)]
    pub wait: bool,

    /// Max seconds to wait when `--wait` is set (default: 3600)
    #[arg(long, requires = "wait")]
    pub wait_timeout_secs: Option<u64>,
}

pub async fn handle(
    cmd: ScanCommand,
    client: &RommClient,
    format: OutputFormat,
    interrupt: Option<InterruptContext>,
) -> Result<()> {
    let slugs: Vec<String> = cmd
        .platform
        .as_deref()
        .map(|s| {
            s.split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let task_kwargs = if slugs.is_empty() {
        None
    } else {
        Some(json!({ "platform_slugs": slugs }))
    };

    let cache_invalidate = if cmd.wait {
        match cmd.platform.as_deref() {
            Some(p) if !p.trim().is_empty() && !p.contains(',') => {
                let service = PlatformService::new(client);
                let platforms = service.list_platforms().await?;
                match services::resolve_platform_id_from_list(p.trim(), &platforms) {
                    Ok(pid) => ScanCacheInvalidate::Platform(pid),
                    Err(_) => ScanCacheInvalidate::AllPlatforms,
                }
            }
            _ => ScanCacheInvalidate::AllPlatforms,
        }
    } else {
        ScanCacheInvalidate::None
    };

    let options = ScanLibraryOptions {
        wait: cmd.wait,
        wait_timeout: Duration::from_secs(cmd.wait_timeout_secs.unwrap_or(3600)),
        cache_invalidate,
        task_kwargs,
    };
    run_scan_library_flow(client, options, format, interrupt.as_ref()).await
}
