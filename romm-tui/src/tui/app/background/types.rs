//! Background task completion types for [`crate::tui::app::App`].

use std::path::PathBuf;
use std::time::Instant;

use crate::core::cache::RomCacheKey;
use crate::core::startup_library_snapshot;
use crate::endpoints::device::DeviceSchema;
use crate::endpoints::roms::GetRoms;
use crate::endpoints::sync::SyncSessionSchema;
use crate::types::{Collection, Platform, RomList, SaveMetadata};
use crate::update::UpdateStatus;

/// Result of a background library metadata refresh (generation-guarded).
#[derive(Debug)]
pub(crate) struct LibraryMetadataRefreshDone {
    pub(crate) gen: u64,
    pub(crate) platforms: Vec<Platform>,
    pub(crate) collections: Vec<Collection>,
    pub(crate) collection_digest: Vec<startup_library_snapshot::CollectionDigestEntry>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct CollectionPrefetchDone {
    pub(crate) key: RomCacheKey,
    pub(crate) expected: u64,
    pub(crate) roms: Option<RomList>,
    pub(crate) warning: Option<String>,
}

#[derive(Debug)]
pub(crate) enum RomLoadEvent {
    Batch(RomList),
    Failed(String),
    Complete,
}

/// Background primary ROM list fetch (deferred load path). Generation-guarded against stale completions.
#[derive(Debug)]
pub(crate) struct RomLoadDone {
    pub(crate) gen: u64,
    pub(crate) key: Option<RomCacheKey>,
    pub(crate) expected: u64,
    pub(crate) event: RomLoadEvent,
    pub(crate) context: &'static str,
    pub(crate) started: Instant,
}

#[derive(Debug)]
pub(crate) enum SearchLoadEvent {
    Batch(RomList),
    Failed(String),
    Complete,
}

#[derive(Debug)]
pub(crate) struct SearchLoadDone {
    pub(crate) query: String,
    pub(crate) event: SearchLoadEvent,
}

#[derive(Debug)]
pub(crate) struct CoverLoadDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<image::DynamicImage, String>,
}

#[derive(Debug)]
pub(crate) struct SaveListDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<Vec<SaveMetadata>, String>,
}

#[derive(Debug)]
pub(crate) struct SaveUploadDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<(), String>,
}

#[derive(Debug)]
pub(crate) struct SaveDownloadDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<PathBuf, String>,
}

#[derive(Debug)]
pub(crate) struct DeviceListDone {
    pub(crate) result: Result<Vec<DeviceSchema>, String>,
}

#[derive(Debug)]
pub(crate) struct PlatformListDone {
    pub(crate) result: Result<Vec<Platform>, String>,
}

#[derive(Debug)]
pub(crate) struct SyncPushPullDone {
    pub(crate) result: Result<SyncSessionSchema, String>,
}

pub(crate) struct StartupUpdatePrompt {
    pub(crate) status: UpdateStatus,
    pub(crate) updating: bool,
}

/// Deferred primary ROM load: cache key, API request, expected count, context label, start time.
pub(crate) type DeferredLoadRoms = (
    Option<RomCacheKey>,
    Option<GetRoms>,
    u64,
    &'static str,
    Instant,
);

#[derive(Debug)]
pub(crate) struct LibraryUploadComplete {
    pub(crate) platform_id: u64,
    pub(crate) scan_after: bool,
}
