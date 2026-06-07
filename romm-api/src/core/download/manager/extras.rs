use std::sync::Arc;

use crate::client::RommClient;
use crate::core::extras::DownloadTarget;
use crate::error::DownloadError;
use crate::types::Rom;

use super::super::extras_job::{finalize_extras_job_status, ExtrasItemResult, ExtrasJob};
use super::super::paths::resolve_download_directory;
use super::super::transfer::{download_target_with_fallback, prepare_download_target_destination};
use super::DownloadManager;

impl DownloadManager {
    pub fn start_extras_download(
        &self,
        rom: &Rom,
        selected: Vec<DownloadTarget>,
        client: RommClient,
        configured_download_dir: Option<&str>,
    ) -> Result<(), DownloadError> {
        if selected.is_empty() {
            return Err(DownloadError::NoExtrasTargets);
        }

        let _ = resolve_download_directory(configured_download_dir)?;

        let platform = rom
            .platform_display_name
            .as_deref()
            .or(rom.platform_custom_name.as_deref())
            .unwrap_or("—")
            .to_string();

        let total_items = selected.len();
        let job = ExtrasJob::new(rom.id, rom.name.clone(), platform, total_items);
        let job_id = job.id;

        match self.extras_jobs.lock() {
            Ok(mut jobs) => jobs.push(job),
            Err(err) => {
                eprintln!("warning: extras job list lock poisoned: {}", err);
                return Err(DownloadError::ExtrasJobListPoisoned(err.to_string()));
            }
        }

        let extras_jobs = self.extras_jobs.clone();
        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(4));
            let mut handles = Vec::new();

            for target in selected {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break,
                };
                let client = client.clone();
                let extras_jobs = extras_jobs.clone();
                handles.push(tokio::spawn(async move {
                    let mut on_progress = |_r: u64, _t: u64| {};
                    let download_result = match prepare_download_target_destination(&target).await {
                        Ok(true) => Ok(()),
                        Ok(false) => {
                            download_target_with_fallback(
                                &client,
                                &target,
                                |_, _| false,
                                &mut on_progress,
                            )
                            .await
                        }
                        Err(err) => Err(err),
                    };

                    drop(permit);

                    let (ok, err) = match download_result {
                        Ok(()) => (true, None),
                        Err(e) => (false, Some(e.to_string())),
                    };

                    let item = ExtrasItemResult {
                        title: target.title.clone(),
                        kind: target.kind,
                        ok,
                        error: err,
                    };

                    if let Ok(mut list) = extras_jobs.lock() {
                        if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                            j.completed_items = j.completed_items.saturating_add(1);
                            j.item_results.push(item);
                            if j.completed_items >= j.total_items {
                                j.status = finalize_extras_job_status(&j.item_results);
                            }
                        }
                    }
                }));
            }

            for h in handles {
                let _ = h.await;
            }
        });

        Ok(())
    }
}
