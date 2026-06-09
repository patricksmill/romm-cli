//! Apply drained background work to [`super::super::App`] state.

use romm_api::core::library_scan::ScanCacheInvalidate;

use super::super::event::BackgroundAction;
use super::super::{App, AppScreen};
use super::types::{RomLoadDone, RomLoadEvent};

impl App {
    /// Apply background-only actions synchronously (tests and legacy poll path).
    pub(in crate::tui::app) fn apply_background(&mut self, action: BackgroundAction) {
        match action {
            BackgroundAction::LibraryMetadataRefresh(done) => {
                self.apply_library_metadata_refresh(done);
            }
            BackgroundAction::RomLoad(done) => self.apply_rom_load_complete(done),
            BackgroundAction::CollectionPrefetch(done) => {
                self.apply_collection_prefetch_complete(done);
            }
            BackgroundAction::SearchLoad(done) => self.apply_search_load_complete(done),
            BackgroundAction::CoverLoad(done) => self.apply_cover_load_complete(done),
            BackgroundAction::SaveList(done) => self.apply_save_list_complete(done),
            BackgroundAction::SaveUpload(done) => self.apply_save_upload_complete(done),
            BackgroundAction::SaveDownload(done) => self.apply_save_download_complete(done),
            BackgroundAction::DeviceList(done) => self.apply_device_list_complete(done),
            BackgroundAction::PlatformList(done) => self.apply_platform_list_complete(done),
            BackgroundAction::SyncPushPull(done) => self.apply_sync_push_pull_complete(done),
            BackgroundAction::LibraryUploadProgress { uploaded, total } => {
                self.apply_library_upload_progress(uploaded, total);
            }
            BackgroundAction::LibraryUploadDone(result) => {
                self.apply_library_upload_complete(result)
            }
            BackgroundAction::LibraryScanDone(result) => self.apply_library_scan_complete(result),
            BackgroundAction::DrivePrefetch => self.drive_collection_prefetch_scheduler(),
            BackgroundAction::PollFooterClear => {
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    lib.poll_footer_clear();
                }
            }
        }
    }

    fn apply_rom_load_complete(&mut self, done: RomLoadDone) {
        if !super::super::rom_load::primary_rom_load_result_is_current(done.gen, self.rom_load_gen)
        {
            return;
        }
        let AppScreen::LibraryBrowse(ref mut lib) = self.screen else {
            return;
        };
        if !super::super::rom_load::primary_rom_load_result_matches_selection(lib, &done.key) {
            if matches!(done.event, RomLoadEvent::Complete | RomLoadEvent::Failed(_)) {
                lib.set_rom_loading(false);
            }
            tracing::debug!(
                "rom-list-render skipped stale completion context={}",
                done.context
            );
            return;
        }
        match done.event {
            RomLoadEvent::Batch(roms) => {
                if let Some(ref k) = done.key {
                    self.rom_cache
                        .insert(k.clone(), roms.clone(), done.expected);
                }
                lib.set_roms(roms);
                tracing::debug!(
                    "rom-list-render batch context={} latency_ms={}",
                    done.context,
                    done.started.elapsed().as_millis()
                );
            }
            RomLoadEvent::Failed(e) => {
                lib.set_metadata_footer(Some(format!("Could not load games: {e}")));
                lib.set_rom_loading(false);
            }
            RomLoadEvent::Complete => {
                lib.set_rom_loading(false);
            }
        }
    }

    fn apply_collection_prefetch_complete(&mut self, done: super::types::CollectionPrefetchDone) {
        self.collection_prefetch_inflight_keys.remove(&done.key);
        if let Some(roms) = done.roms {
            self.rom_cache.insert(done.key, roms, done.expected);
        } else if let Some(warning) = done.warning {
            tracing::debug!("{warning}");
        }
    }

    fn apply_search_load_complete(&mut self, done: super::types::SearchLoadDone) {
        use super::types::SearchLoadEvent;
        if let AppScreen::Search(ref mut search) = self.screen {
            match done.event {
                SearchLoadEvent::Batch(roms) => {
                    search.set_results_for_query(done.query, roms);
                }
                SearchLoadEvent::Failed(err) => {
                    search.loading = false;
                    self.global_error = Some(err);
                }
                SearchLoadEvent::Complete => {
                    search.loading = false;
                }
            }
        }
    }

    fn apply_cover_load_complete(&mut self, done: super::types::CoverLoadDone) {
        if let AppScreen::GameDetail(detail) = &mut self.screen {
            if detail.rom.id != done.rom_id {
                return;
            }
            match done.result {
                Ok(image) => detail.apply_cover_image(image),
                Err(err) => detail.apply_cover_error(format!(
                    "Cover failed: {}",
                    crate::tui::utils::truncate(&err, 120)
                )),
            }
        }
    }

    fn apply_save_list_complete(&mut self, done: super::types::SaveListDone) {
        if let AppScreen::GameDetail(detail) = &mut self.screen {
            if detail.rom.id == done.rom_id {
                match done.result {
                    Ok(rows) => detail.apply_saves(rows),
                    Err(e) => detail.apply_saves_error(e),
                }
            }
        }
    }

    fn apply_save_upload_complete(&mut self, done: super::types::SaveUploadDone) {
        use std::time::{Duration, Instant};
        if let AppScreen::GameDetail(detail) = &mut self.screen {
            if detail.rom.id == done.rom_id {
                match done.result {
                    Ok(()) => {
                        detail.message = Some("Save uploaded. Refreshing saves...".into());
                        detail.message_clear_at = Some(Instant::now() + Duration::from_secs(3));
                        self.spawn_save_list_worker(done.rom_id);
                    }
                    Err(e) => {
                        detail.message = Some(format!("Save upload failed: {e}"));
                        detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                    }
                }
            }
        }
    }

    fn apply_save_download_complete(&mut self, done: super::types::SaveDownloadDone) {
        use std::time::{Duration, Instant};
        if let AppScreen::GameDetail(detail) = &mut self.screen {
            if detail.rom.id == done.rom_id {
                match done.result {
                    Ok(path) => {
                        detail.message = Some(format!("Save downloaded: {}", path.display()));
                        detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                        self.spawn_save_list_worker(done.rom_id);
                    }
                    Err(e) => {
                        detail.message = Some(format!("Save download failed: {e}"));
                        detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                    }
                }
            }
        }
    }

    fn apply_device_list_complete(&mut self, done: super::types::DeviceListDone) {
        use crate::tui::theme::MessageTone;
        if let AppScreen::Settings(settings) = &mut self.screen {
            match done.result {
                Ok(devices) => {
                    settings.set_devices(devices);
                    settings.message = None;
                }
                Err(e) => {
                    settings.set_device_error(e.clone());
                    settings.message =
                        Some((format!("Device load failed: {e}"), MessageTone::Error));
                }
            }
        }
    }

    fn apply_platform_list_complete(&mut self, done: super::types::PlatformListDone) {
        use crate::tui::theme::MessageTone;
        if let AppScreen::Settings(settings) = &mut self.screen {
            match done.result {
                Ok(platforms) => {
                    settings.set_console_platforms(platforms);
                    settings.message = None;
                }
                Err(e) => {
                    settings.set_console_platform_error(e.clone());
                    settings.message =
                        Some((format!("Platform load failed: {e}"), MessageTone::Error));
                }
            }
        }
    }

    fn apply_sync_push_pull_complete(&mut self, done: super::types::SyncPushPullDone) {
        use crate::tui::theme::MessageTone;
        if let AppScreen::Settings(settings) = &mut self.screen {
            settings.sync_inflight = false;
            match done.result {
                Ok(session) => {
                    settings.message = Some((
                        format!("Sync session #{}: {}", session.id, session.status),
                        MessageTone::Success,
                    ));
                }
                Err(e) => {
                    settings.message = Some((format!("Sync failed: {e}"), MessageTone::Error));
                }
            }
        }
    }

    fn apply_library_upload_progress(&mut self, uploaded: u64, total: u64) {
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            lib.set_metadata_footer(Some(format!(
                "Uploading {} / {}…",
                Self::format_upload_bytes(uploaded),
                Self::format_upload_bytes(total)
            )));
        }
    }

    fn apply_library_upload_complete(
        &mut self,
        result: Result<super::types::LibraryUploadComplete, String>,
    ) {
        self.library_upload_done_rx = None;
        self.library_upload_progress_rx = None;
        self.library_upload_inflight = false;
        match result {
            Ok(done) => {
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    if done.scan_after {
                        lib.set_metadata_footer(Some(
                            "Upload complete. Starting library scan…".into(),
                        ));
                        self.spawn_library_rescan_worker(ScanCacheInvalidate::Platform(
                            done.platform_id,
                        ));
                    } else {
                        lib.set_metadata_footer(Some("Upload complete.".into()));
                    }
                }
            }
            Err(e) => {
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    lib.set_metadata_footer(Some(format!("Upload failed: {e}")));
                } else {
                    self.global_error = Some(format!("Upload failed: {e}"));
                }
            }
        }
    }

    fn apply_library_scan_complete(&mut self, result: Result<(), String>) {
        self.library_scan_rx = None;
        self.library_scan_inflight = false;
        match result {
            Ok(()) => self.on_library_scan_completed_success(),
            Err(e) => {
                self.library_scan_pending_invalidate = None;
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    lib.set_metadata_footer(Some(format!("Library scan failed: {e}")));
                } else {
                    self.global_error = Some(format!("Library scan failed: {e}"));
                }
            }
        }
    }
}
