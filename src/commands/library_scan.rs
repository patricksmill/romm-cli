//! Shared `scan_library` task trigger and optional wait (used by `roms upload` and `scan`).

use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use indicatif::ProgressBar;
use serde_json::Value;

use crate::client::RommClient;

use super::OutputFormat;

pub const SCAN_LIBRARY_TASK_NAME: &str = "scan_library";

/// Options for starting a library scan and optionally blocking until it finishes.
#[derive(Clone, Debug)]
pub struct ScanLibraryOptions {
    pub wait: bool,
    pub wait_timeout: Duration,
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
/// On success returns the last status JSON (typically `status` == `finished`).
pub async fn wait_for_scan_task(
    client: &RommClient,
    task_id: &str,
    timeout: Duration,
) -> Result<Value> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(format!("Waiting for library scan (task {task_id})…"));

    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline {
            pb.finish_and_clear();
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

        pb.set_message(format!("Library scan: {st}…"));

        if is_terminal_status(st) {
            pb.finish_and_clear();
            if is_success_status(st) {
                return Ok(body);
            }
            anyhow::bail!("library scan task ended with status {st:?}: {body}");
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Start a library scan; optionally wait. Prints human text or JSON per `format`.
pub async fn run_scan_library_flow(
    client: &RommClient,
    options: ScanLibraryOptions,
    format: OutputFormat,
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
        let final_body = wait_for_scan_task(client, &start.task_id, options.wait_timeout).await?;
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
