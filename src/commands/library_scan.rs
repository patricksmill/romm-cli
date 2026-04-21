//! Shared `scan_library` task trigger and optional wait (used by `roms upload` and `scan`).

use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use indicatif::ProgressBar;
use serde_json::Value;

use crate::client::RommClient;
use crate::core::cache::{RomCache, RomCacheKey};
use crate::core::interrupt::{cancelled_error, InterruptContext};

use super::OutputFormat;

pub const SCAN_LIBRARY_TASK_NAME: &str = "scan_library";

/// After a successful `--wait`, optionally drop stale entries from the on-disk ROM list cache.
#[derive(Clone, Debug, Default)]
pub enum ScanCacheInvalidate {
    #[default]
    None,
    /// Clear the cached list for this platform (e.g. after `roms upload … --scan --wait`).
    Platform(u64),
    /// Clear every platform entry (full `scan_library` scope).
    AllPlatforms,
}

/// Options for starting a library scan and optionally blocking until it finishes.
#[derive(Clone, Debug)]
pub struct ScanLibraryOptions {
    pub wait: bool,
    pub wait_timeout: Duration,
    pub cache_invalidate: ScanCacheInvalidate,
}

fn apply_cache_invalidate(inv: &ScanCacheInvalidate) {
    match inv {
        ScanCacheInvalidate::None => {}
        ScanCacheInvalidate::Platform(pid) => {
            let mut c = RomCache::load();
            c.remove(&RomCacheKey::Platform(*pid));
        }
        ScanCacheInvalidate::AllPlatforms => {
            let mut c = RomCache::load();
            c.remove_all_platform_entries();
        }
    }
}

#[derive(Debug)]
pub struct ScanLibraryStart {
    pub task_id: String,
    pub initial_status: String,
    pub raw: Value,
}

/// POST `scan_library` with no kwargs (RomM task accepts no `platform_id`; see docs).
pub async fn start_scan_library(client: &RommClient) -> Result<ScanLibraryStart> {
    let raw = client
        .run_task(SCAN_LIBRARY_TASK_NAME, None)
        .await
        .context("failed to start scan_library task")?;
    let task_id = raw
        .get("task_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            anyhow!(
                "scan response missing task_id (unexpected server response): {}",
                raw
            )
        })?
        .to_string();
    let initial_status = raw
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    Ok(ScanLibraryStart {
        task_id,
        initial_status,
        raw,
    })
}

fn status_from_json(v: &Value) -> Option<&str> {
    v.get("status").and_then(|s| s.as_str())
}

fn is_terminal_status(status: &str) -> bool {
    status.eq_ignore_ascii_case("finished")
        || status.eq_ignore_ascii_case("failed")
        || status.eq_ignore_ascii_case("stopped")
        || status.eq_ignore_ascii_case("canceled")
        || status.eq_ignore_ascii_case("cancelled")
}

fn is_success_status(status: &str) -> bool {
    status.eq_ignore_ascii_case("finished")
}

/// Poll `GET /api/tasks/{task_id}` every 2 seconds until terminal state or timeout.
/// `on_status` is invoked with each non-terminal status string (may be empty on parse miss).
/// On success returns the last status JSON (typically `status` == `finished`).
pub async fn wait_for_task_terminal(
    client: &RommClient,
    task_id: &str,
    timeout: Duration,
    interrupt: Option<&InterruptContext>,
    mut on_status: impl FnMut(&str),
) -> Result<Value> {
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline {
            anyhow::bail!(
                "timed out waiting for library scan task {} after {:?}",
                task_id,
                timeout
            );
        }

        let body = client
            .get_task_status(task_id)
            .await
            .with_context(|| format!("failed to poll task {task_id}"))?;
        let st = status_from_json(&body).unwrap_or("");

        if is_terminal_status(st) {
            if is_success_status(st) {
                return Ok(body);
            }
            anyhow::bail!("library scan task ended with status {st:?}: {body}");
        }

        on_status(st);
        if let Some(ctx) = interrupt {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(2)) => {},
                _ = ctx.cancelled() => return Err(cancelled_error()),
            }
        } else {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

/// CLI: poll task status with an `indicatif` spinner (do not use under the TUI alternate screen).
pub async fn wait_for_scan_task(
    client: &RommClient,
    task_id: &str,
    timeout: Duration,
    interrupt: Option<&InterruptContext>,
) -> Result<Value> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(format!("Waiting for library scan (task {task_id})…"));

    let result = wait_for_task_terminal(client, task_id, timeout, interrupt, |st| {
        pb.set_message(format!("Library scan: {st}…"));
    })
    .await;

    pb.finish_and_clear();
    result
}

/// Start a library scan; optionally wait. Prints human text or JSON per `format`.
pub async fn run_scan_library_flow(
    client: &RommClient,
    options: ScanLibraryOptions,
    format: OutputFormat,
    interrupt: Option<&InterruptContext>,
) -> Result<()> {
    match format {
        OutputFormat::Text => println!("Triggering library scan..."),
        OutputFormat::Json => {}
    }

    let start = start_scan_library(client).await?;

    match format {
        OutputFormat::Text => println!(
            "Scan started: task_id={}, status={}",
            start.task_id, start.initial_status
        ),
        OutputFormat::Json if !options.wait => {
            println!("{}", serde_json::to_string_pretty(&start.raw)?);
        }
        OutputFormat::Json => {}
    }

    if options.wait {
        let final_body =
            wait_for_scan_task(client, &start.task_id, options.wait_timeout, interrupt).await?;
        apply_cache_invalidate(&options.cache_invalidate);
        match format {
            OutputFormat::Text => println!("Library scan finished successfully."),
            OutputFormat::Json => {
                let mut out = start.raw;
                if let Value::Object(ref mut m) = out {
                    m.insert("final_status".into(), final_body);
                }
                println!("{}", serde_json::to_string_pretty(&out)?);
            }
        }
    }

    Ok(())
}
