//! CLI presentation wrappers for shared library scan helpers in `romm_api::core::library_scan`.

use std::time::Duration;

use anyhow::Result;
use indicatif::ProgressBar;
use serde_json::Value;

use romm_api::client::RommClient;
use romm_api::core::interrupt::InterruptContext;
use romm_api::core::library_scan::{
    apply_disk_cache_invalidate, start_scan_library, wait_for_task_terminal,
};

use super::OutputFormat;
use crate::cli_presentation::CliPresentation;

pub use romm_api::core::library_scan::{
    ScanCacheInvalidate, ScanLibraryOptions, ScanLibraryStart, SCAN_LIBRARY_TASK_NAME,
};

/// CLI: poll task status with an `indicatif` spinner when text mode allows progress.
pub async fn wait_for_scan_task(
    client: &RommClient,
    task_id: &str,
    timeout: Duration,
    interrupt: Option<&InterruptContext>,
    presentation: CliPresentation,
) -> Result<Value> {
    if presentation.is_json() || !presentation.shows_progress() {
        return wait_for_task_terminal(client, task_id, timeout, interrupt, |_| {}).await;
    }

    let pb = ProgressBar::new_spinner();
    pb.set_draw_target(presentation.progress_draw_target());
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(format!("Waiting for library scan (task {task_id})…"));

    let result = wait_for_task_terminal(client, task_id, timeout, interrupt, |st| {
        pb.set_message(format!("Library scan: {st}…"));
    })
    .await;

    pb.finish_and_clear();
    result
}

/// Start a library scan; optionally wait. Prints human text or JSON per `presentation`.
pub async fn run_scan_library_flow(
    client: &RommClient,
    options: ScanLibraryOptions,
    presentation: CliPresentation,
    interrupt: Option<&InterruptContext>,
) -> Result<()> {
    let format = presentation.format;
    match format {
        OutputFormat::Text => println!("Triggering library scan..."),
        OutputFormat::Json => {}
    }

    let start = start_scan_library(client, options.task_kwargs.clone()).await?;

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
        let final_body = wait_for_scan_task(
            client,
            &start.task_id,
            options.wait_timeout,
            interrupt,
            presentation,
        )
        .await?;
        apply_disk_cache_invalidate(&options.cache_invalidate);
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
