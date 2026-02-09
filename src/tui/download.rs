/// Shared state for active and recent downloads (name, platform, progress, status).
use std::sync::atomic::{AtomicUsize, Ordering};

/// Status of a single download job.
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Downloading,
    Done,
    Error(String),
}

/// One download job (ROM) for display in the Download screen.
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

    /// Progress as percentage 0..=100 for display.
    pub fn percent(&self) -> u16 {
        (self.progress * 100.0).round().min(100.0) as u16
    }
}
