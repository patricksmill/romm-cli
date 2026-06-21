//! Download state and management.
//!
//! `DownloadJob` holds per-download progress/status.
//! `DownloadManager` owns the shared job list and spawns background tasks.
//!
//! This module handles downloading files from the RomM server, including resume
//! support and progress tracking.

mod extras_job;
mod job;
mod manager;
mod paths;
mod transfer;

#[cfg(test)]
mod tests;

pub use extras_job::*;
pub use job::*;
pub use manager::DownloadManager;
pub use paths::*;
pub use transfer::{download_target_with_fallback, prepare_download_target_destination};
