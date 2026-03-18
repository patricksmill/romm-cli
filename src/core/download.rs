//! Download state and management.
//!
//! `DownloadJob` holds per-download progress/status.
//! `DownloadManager` owns the shared job list and spawns background tasks.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::client::RommClient;
use crate::core::utils;
use crate::types::Rom;

// ---------------------------------------------------------------------------
// Job status / data
// ---------------------------------------------------------------------------

/// High-level status of a single download.
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Downloading,
    Done,
    Error(String),
}

/// A single background download job (for one ROM).
#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub id: usize,
    pub rom_id: u64,
    pub name: String,
    pub platform: String,
    /// 0.0 ..= 1.0
    pub progress: f64,
    pub status: DownloadStatus,
}

static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(0);

impl DownloadJob {
    /// Construct a new job in the `Downloading` state.
    pub fn new(rom_id: u64, name: String, platform: String) -> Self {
        Self {
            id: NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed),
            rom_id,
            name,
            platform,
            progress: 0.0,
            status: DownloadStatus::Downloading,
        }
    }

    /// Progress as percentage 0..=100.
    pub fn percent(&self) -> u16 {
        (self.progress * 100.0).round().min(100.0) as u16
    }
}

// ---------------------------------------------------------------------------
// Manager
// ---------------------------------------------------------------------------

/// Owns the shared download list and spawns background download tasks.
///
/// Frontends only need an `Arc<Mutex<Vec<DownloadJob>>>` to inspect jobs.
#[derive(Clone)]
pub struct DownloadManager {
    jobs: Arc<Mutex<Vec<DownloadJob>>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Shared handle for observers (TUI, GUI, tests) to inspect jobs.
    pub fn shared(&self) -> Arc<Mutex<Vec<DownloadJob>>> {
        self.jobs.clone()
    }

    /// Start downloading `rom` in the background; returns immediately.
    ///
    /// Progress updates are pushed into the shared `jobs` list so that
    /// any frontend can render them.
    pub fn start_download(&self, rom: &Rom, client: RommClient) {
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
        match self.jobs.lock() {
            Ok(mut jobs) => jobs.push(job),
            Err(err) => {
                eprintln!("warning: download job list lock poisoned: {}", err);
                return;
            }
        }

        let jobs = self.jobs.clone();
        tokio::spawn(async move {
            let save_dir = Path::new("./downloads");
            if let Err(err) = tokio::fs::create_dir_all(save_dir).await {
                eprintln!(
                    "warning: failed to create download directory {:?}: {}",
                    save_dir, err
                );
            }
            let base = utils::sanitize_filename(&fs_name);
            let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(&base);
            let save_path = save_dir.join(format!("{}.zip", stem));

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

            match client.download_rom(rom_id, &save_path, on_progress).await {
                Ok(()) => {
                    if let Ok(mut list) = jobs.lock() {
                        if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                            j.status = DownloadStatus::Done;
                            j.progress = 1.0;
                        }
                    }
                }
                Err(e) => {
                    if let Ok(mut list) = jobs.lock() {
                        if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                            j.status = DownloadStatus::Error(e.to_string());
                        }
                    }
                }
            }
        });
    }
}
