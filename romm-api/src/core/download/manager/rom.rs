use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::client::RommClient;
use crate::config::RomsLayoutConfig;
use crate::core::extras::{build_base_rom_file_targets, DownloadTarget};
use crate::core::interrupt::is_cancelled_download;
use crate::core::utils;
use crate::error::DownloadError;
use crate::types::Rom;

use super::super::job::{DownloadJob, DownloadStatus};
use super::super::paths::{resolve_console_roms_dir, resolve_download_directory};
use super::super::transfer::{
    download_target_with_fallback, finalize_download, prepare_download_target_destination,
    sanitized_final_filename, FinalizeResult,
};
use super::DownloadManager;

struct RomDownloadTask {
    client: RommClient,
    jobs: Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    rom_id: u64,
    fs_name: String,
    final_name: String,
    save_dir: PathBuf,
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
        let fs_name = rom.fs_name.clone();
        let final_name = sanitized_final_filename(&rom.fs_name, rom.id);
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
            fs_name,
            final_name,
            save_dir,
            console_dir,
            base_targets,
        };
        tokio::spawn(run_rom_download_task(task));
        Ok(())
    }
}

async fn run_rom_download_task(task: RomDownloadTask) {
    let temp_root = task.save_dir.join(".tmp");
    if let Err(err) = tokio::fs::create_dir_all(&temp_root).await {
        set_job_status(
            &task.jobs,
            task.job_id,
            DownloadStatus::Error(format!(
                "Could not create temp directory {}: {err}",
                temp_root.display()
            )),
        );
        return;
    }

    let final_path = task.console_dir.join(task.final_name.clone());
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

    if !task.base_targets.is_empty() {
        download_base_targets(&task.client, &task.jobs, task.job_id, &task.base_targets).await;
        return;
    }

    download_primary_rom(
        &task.client,
        &task.jobs,
        task.job_id,
        task.rom_id,
        &task.fs_name,
        &temp_root,
        &final_path,
    )
    .await;
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

async fn download_primary_rom(
    client: &RommClient,
    jobs: &Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    rom_id: u64,
    fs_name: &str,
    temp_root: &std::path::Path,
    final_path: &std::path::Path,
) {
    if final_path.exists() {
        finish_job(jobs, job_id, DownloadStatus::SkippedAlreadyExists);
        return;
    }

    let temp_name = format!(
        "rom-{}-{}-{}.part",
        rom_id,
        utils::sanitize_filename(fs_name),
        job_id
    );
    let temp_path = temp_root.join(temp_name);
    let progress_jobs = jobs.clone();
    let on_progress = move |received: u64, total: u64| {
        let p = if total > 0 {
            received as f64 / total as f64
        } else {
            0.0
        };

        set_job_progress(&progress_jobs, job_id, p);
    };

    let download_result = client.download_rom(rom_id, &temp_path, on_progress).await;
    if download_result.is_err() {
        let _ = tokio::fs::remove_file(&temp_path).await;
    }
    match download_result {
        Ok(()) => match finalize_download(&temp_path, final_path).await {
            Ok(FinalizeResult::Done) => {
                finish_job(jobs, job_id, DownloadStatus::Done);
            }
            Ok(FinalizeResult::SkippedAlreadyExists) => {
                finish_job(jobs, job_id, DownloadStatus::SkippedAlreadyExists);
            }
            Err(err) => {
                let _ = tokio::fs::remove_file(&temp_path).await;
                set_job_status(
                    jobs,
                    job_id,
                    DownloadStatus::FinalizeFailed(err.to_string()),
                );
            }
        },
        Err(e) => {
            if is_cancelled_download(&e) {
                set_job_status(jobs, job_id, DownloadStatus::Cancelled);
            } else {
                set_job_status(jobs, job_id, DownloadStatus::Error(e.to_string()));
            }
        }
    }
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
