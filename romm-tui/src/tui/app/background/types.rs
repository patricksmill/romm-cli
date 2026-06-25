//! Background task completion types for [`crate::tui::app::App`].

use std::path::PathBuf;
use std::time::Instant;

use romm_api::core::cache::RomCacheKey;
use romm_api::core::startup_library_snapshot;
use romm_api::endpoints::device::DeviceSchema;
use romm_api::endpoints::roms::GetRoms;
use romm_api::endpoints::sync::SyncSessionSchema;
use romm_api::error::RommError;
use romm_api::types::{Collection, Platform, RomList, SaveMetadata};
use romm_api::update::UpdateStatus;

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
    Failed(RommError),
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
    Failed(RommError),
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
    pub(crate) result: Result<image::DynamicImage, RommError>,
}

#[derive(Debug)]
pub(crate) struct SaveListDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<Vec<SaveMetadata>, RommError>,
}

#[derive(Debug)]
pub(crate) struct SaveUploadDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<(), RommError>,
}

#[derive(Debug)]
pub(crate) struct SaveDownloadDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<PathBuf, RommError>,
}

#[derive(Debug)]
pub(crate) struct DeviceListDone {
    pub(crate) result: Result<Vec<DeviceSchema>, RommError>,
}

#[derive(Debug)]
pub(crate) struct PlatformListDone {
    pub(crate) result: Result<Vec<Platform>, RommError>,
}

#[derive(Debug)]
pub(crate) struct SyncPushPullDone {
    pub(crate) result: Result<SyncSessionSchema, RommError>,
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
