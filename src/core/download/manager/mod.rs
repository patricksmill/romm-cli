//! Shared download job list and background task spawning.

mod extras;
mod rom;

use std::sync::{Arc, Mutex};

use super::extras_job::ExtrasJob;
use super::job::DownloadJob;

/// Owns the shared download list and spawns background download tasks.
///
/// Frontends only need an `Arc<Mutex<Vec<DownloadJob>>>` to inspect jobs.
#[derive(Clone)]
pub struct DownloadManager {
    pub(super) jobs: Arc<Mutex<Vec<DownloadJob>>>,
    pub(super) extras_jobs: Arc<Mutex<Vec<ExtrasJob>>>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            extras_jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Shared handle for observers (TUI, GUI, tests) to inspect jobs.
    pub fn shared(&self) -> Arc<Mutex<Vec<DownloadJob>>> {
        self.jobs.clone()
    }

    /// Shared extras jobs (composite batches).
    pub fn shared_extras(&self) -> Arc<Mutex<Vec<ExtrasJob>>> {
        self.extras_jobs.clone()
    }
}
