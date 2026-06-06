use crate::client::RommClient;
use crate::config::RomsLayoutConfig;
use crate::core::extras::build_base_rom_file_targets;
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
        let jobs = self.jobs.clone();
        tokio::spawn(async move {
            let temp_root = save_dir.join(".tmp");
            if let Err(err) = tokio::fs::create_dir_all(&temp_root).await {
                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.status = DownloadStatus::Error(format!(
                            "Could not create temp directory {}: {err}",
                            temp_root.display()
                        ));
                    }
                }
                return;
            }

            let final_path = console_dir.join(final_name.clone());
            if let Err(err) = tokio::fs::create_dir_all(&console_dir).await {
                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.status = DownloadStatus::Error(format!(
                            "Could not create console directory {}: {err}",
                            console_dir.display()
                        ));
                    }
                }
                return;
            }

            let base_targets = base_targets;
            if !base_targets.is_empty() {
                let total_targets = base_targets.len() as f64;
                for (idx, target) in base_targets.iter().enumerate() {
                    let client = client.clone();
                    let mut progress = {
                        let jobs = jobs.clone();
                        move |received: u64, total: u64| {
                            let file_ratio = if total > 0 {
                                received as f64 / total as f64
                            } else {
                                0.0
                            };
                            let total_ratio = ((idx as f64) + file_ratio) / total_targets;
                            if let Ok(mut list) = jobs.lock() {
                                if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                    j.progress = total_ratio.min(1.0);
                                }
                            }
                        }
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
                            if let Ok(mut list) = jobs.lock() {
                                if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                    j.status = DownloadStatus::Error(err.to_string());
                                }
                            }
                            return;
                        }
                    }
                    if let Err(final_err) =
                        download_target_with_fallback(&client, target, |_, _| false, &mut progress)
                            .await
                    {
                        if let Ok(mut list) = jobs.lock() {
                            if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                j.status = DownloadStatus::Error(final_err.to_string());
                            }
                        }
                        return;
                    }
                }
                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.status = DownloadStatus::Done;
                        j.progress = 1.0;
                    }
                }
                return;
            }

            if final_path.exists() {
                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.status = DownloadStatus::SkippedAlreadyExists;
                        j.progress = 1.0;
                    }
                }
                return;
            }

            let temp_name = format!(
                "rom-{}-{}-{}.part",
                rom_id,
                utils::sanitize_filename(&fs_name),
                job_id
            );
            let temp_path = temp_root.join(temp_name);

            let on_progress = |received: u64, total: u64| {
                let p = if total > 0 {
                    received as f64 / total as f64
                } else {
                    0.0
                };

                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.progress = p;
                    }
                }
            };

            let download_result = client.download_rom(rom_id, &temp_path, on_progress).await;
            if download_result.is_err() {
                let _ = tokio::fs::remove_file(&temp_path).await;
            }
            match download_result {
                Ok(()) => match finalize_download(&temp_path, &final_path).await {
                    Ok(FinalizeResult::Done) => {
                        if let Ok(mut list) = jobs.lock() {
                            if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                j.status = DownloadStatus::Done;
                                j.progress = 1.0;
                            }
                        }
                    }
                    Ok(FinalizeResult::SkippedAlreadyExists) => {
                        if let Ok(mut list) = jobs.lock() {
                            if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                j.status = DownloadStatus::SkippedAlreadyExists;
                                j.progress = 1.0;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        if let Ok(mut list) = jobs.lock() {
                            if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                j.status = DownloadStatus::FinalizeFailed(err.to_string());
                            }
                        }
                    }
                },
                Err(e) => {
                    if let Ok(mut list) = jobs.lock() {
                        if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                            if is_cancelled_download(&e) {
                                j.status = DownloadStatus::Cancelled;
                            } else {
                                j.status = DownloadStatus::Error(e.to_string());
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
