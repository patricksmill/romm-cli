//! Composite extras download job state.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::core::extras::DownloadAssetKind;

/// Outcome of one item inside an [`ExtrasJob`].
#[derive(Debug, Clone)]
pub struct ExtrasItemResult {
    pub title: String,
    pub kind: DownloadAssetKind,
    pub ok: bool,
    pub error: Option<String>,
}

/// Terminal status for a composite extras download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtrasJobStatus {
    Running,
    Done,
    /// Some items failed (`usize` = failure count).
    PartialFailure(usize),
    AllFailed,
}

/// One queued extras batch for a parent ROM (related files + cover + manual).
#[derive(Debug, Clone)]
pub struct ExtrasJob {
    pub id: usize,
    pub rom_id: u64,
    pub name: String,
    pub platform: String,
    pub completed_items: usize,
    pub total_items: usize,
    pub status: ExtrasJobStatus,
    pub item_results: Vec<ExtrasItemResult>,
}

static NEXT_EXTRAS_JOB_ID: AtomicUsize = AtomicUsize::new(0);

impl ExtrasJob {
    pub fn new(rom_id: u64, name: String, platform: String, total_items: usize) -> Self {
        Self {
            id: NEXT_EXTRAS_JOB_ID.fetch_add(1, Ordering::Relaxed),
            rom_id,
            name,
            platform,
            completed_items: 0,
            total_items,
            status: ExtrasJobStatus::Running,
            item_results: Vec::new(),
        }
    }

    /// Progress 0..=100 from completed item count only.
    pub fn percent(&self) -> u16 {
        if self.total_items == 0 {
            return 100;
        }
        ((self.completed_items.saturating_mul(100)) / self.total_items).min(100) as u16
    }
}

pub(crate) fn finalize_extras_job_status(results: &[ExtrasItemResult]) -> ExtrasJobStatus {
    let n = results.len();
    if n == 0 {
        return ExtrasJobStatus::Done;
    }
    let failures = results.iter().filter(|r| !r.ok).count();
    if failures == 0 {
        ExtrasJobStatus::Done
    } else if failures == n {
        ExtrasJobStatus::AllFailed
    } else {
        ExtrasJobStatus::PartialFailure(failures)
    }
}
