use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::client::RommClient;
use crate::config::RomsLayoutConfig;
use crate::core::extras::{build_base_rom_file_targets, DownloadTarget};
use crate::error::DownloadError;
use crate::types::Rom;

use super::super::job::{DownloadJob, DownloadStatus};
use super::super::paths::{resolve_console_roms_dir, resolve_download_directory};
use super::super::transfer::{
    download_target_with_fallback, prepare_download_target_destination,
};
use super::DownloadManager;

struct RomDownloadTask {
    client: RommClient,
    jobs: Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    rom_id: u64,
    console_dir: PathBuf,
    base_targets: Vec<DownloadTarget>,
}

impl DownloadManager {
    pub fn start_download(
        &self,
        rom: &Rom,
        client: RommClient,
        layout: &RomsLayoutConfig,
        configured_download_dir: Option<&str>,
    ) -> Result<(), DownloadError> {
        let platform = rom
            .platform_display_name
            .as_deref()
            .or(rom.platform_custom_name.as_deref())
            .unwrap_or("—")
            .to_string();

        let job = DownloadJob::new(rom.id, rom.name.clone(), platform);
        let job_id = job.id;
        let rom_id = rom.id;
        let rom_for_targets = rom.clone();
        let layout = layout.clone();
        match self.jobs.lock() {
            Ok(mut jobs) => jobs.push(job),
            Err(err) => {
                eprintln!("warning: download job list lock poisoned: {}", err);
                return Err(DownloadError::JobListPoisoned(err.to_string()));
            }
        }

        let save_dir = resolve_download_directory(configured_download_dir)?;
        let console_dir = resolve_console_roms_dir(&layout, &save_dir, rom)?;
        let base_targets = build_base_rom_file_targets(&rom_for_targets, &layout, &save_dir)?;
        let task = RomDownloadTask {
            client,
            jobs: self.jobs.clone(),
            job_id,
            rom_id,
            console_dir,
            base_targets,
        };
        tokio::spawn(run_rom_download_task(task, layout, save_dir));
        Ok(())
    }
}

async fn run_rom_download_task(
    task: RomDownloadTask,
    layout: RomsLayoutConfig,
    save_dir: PathBuf,
) {
    if let Err(err) = tokio::fs::create_dir_all(&task.console_dir).await {
        set_job_status(
            &task.jobs,
            task.job_id,
            DownloadStatus::Error(format!(
                "Could not create console directory {}: {err}",
                task.console_dir.display()
            )),
        );
        return;
    }

    // ROMs from the library list endpoint may have an empty `files` vec because
    // list responses omit per-file records. Always resolve the full file list by
    // preferring what we already have, then falling back to a detail fetch.
    // If the detail also returns no files, we surface a clear error — the
    // /api/roms/download endpoint wraps every file in a ZIP archive regardless
    // of type, which corrupts non-ZIP ROMs, so it is never used here.
    let targets = if !task.base_targets.is_empty() {
        task.base_targets
    } else {
        match fetch_base_targets_from_detail(
            &task.client,
            &task.jobs,
            task.job_id,
            task.rom_id,
            &layout,
            &save_dir,
        )
        .await
        {
            Some(t) => t,
            None => return, // error already recorded
        }
    };

    download_base_targets(&task.client, &task.jobs, task.job_id, &targets).await;
}

/// Fetches the full ROM detail (which includes file records) and builds
/// per-file download targets.
///
/// Returns `None` if the API call fails **or** if no file records exist on
/// the server — the error is recorded in the job before returning.
async fn fetch_base_targets_from_detail(
    client: &RommClient,
    jobs: &Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    rom_id: u64,
    layout: &RomsLayoutConfig,
    save_dir: &std::path::Path,
) -> Option<Vec<DownloadTarget>> {
    use crate::endpoints::roms::GetRom;

    let detailed_rom = match client.call(&GetRom { id: rom_id }).await {
        Ok(r) => r,
        Err(e) => {
            set_job_status(
                jobs,
                job_id,
                DownloadStatus::Error(format!("Failed to fetch ROM detail: {e}")),
            );
            return None;
        }
    };

    match build_base_rom_file_targets(&detailed_rom, layout, save_dir) {
        Ok(targets) if !targets.is_empty() => Some(targets),
        Ok(_) => {
            set_job_status(
                jobs,
                job_id,
                DownloadStatus::Error(
                    "No downloadable file records found for this ROM. \
                     Re-scan the library in RomM to populate file metadata."
                        .to_string(),
                ),
            );
            None
        }
        Err(e) => {
            set_job_status(
                jobs,
                job_id,
                DownloadStatus::Error(format!("Failed to build download targets: {e}")),
            );
            None
        }
    }
}

async fn download_base_targets(
    client: &RommClient,
    jobs: &Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    base_targets: &[DownloadTarget],
) {
    let total_targets = base_targets.len() as f64;
    for (idx, target) in base_targets.iter().enumerate() {
        let progress_jobs = jobs.clone();
        let mut progress = move |received: u64, total: u64| {
            let file_ratio = if total > 0 {
                received as f64 / total as f64
            } else {
                0.0
            };
            let total_ratio = ((idx as f64) + file_ratio) / total_targets;
            set_job_progress(&progress_jobs, job_id, total_ratio.min(1.0));
        };

        match prepare_download_target_destination(target).await {
            Ok(true) => {
                progress(
                    target.expected_size_bytes.unwrap_or(0),
                    target.expected_size_bytes.unwrap_or(0),
                );
                continue;
            }
            Ok(false) => {}
            Err(err) => {
                set_job_status(jobs, job_id, DownloadStatus::Error(err.to_string()));
                return;
            }
        }

        if let Err(final_err) =
            download_target_with_fallback(client, target, |_, _| false, &mut progress).await
        {
            set_job_status(jobs, job_id, DownloadStatus::Error(final_err.to_string()));
            return;
        }
    }
    finish_job(jobs, job_id, DownloadStatus::Done);
}

fn update_download_job<F>(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, update: F)
where
    F: FnOnce(&mut DownloadJob),
{
    if let Ok(mut list) = jobs.lock() {
        if let Some(job) = list.iter_mut().find(|job| job.id == job_id) {
            update(job);
        }
    }
}

fn set_job_progress(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, progress: f64) {
    update_download_job(jobs, job_id, |job| {
        job.progress = progress;
    });
}

fn set_job_status(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, status: DownloadStatus) {
    update_download_job(jobs, job_id, |job| {
        job.status = status;
    });
}

fn finish_job(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, status: DownloadStatus) {
    update_download_job(jobs, job_id, |job| {
        job.status = status;
        job.progress = 1.0;
    });
}
