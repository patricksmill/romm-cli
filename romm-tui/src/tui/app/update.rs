//! Apply semantic [`Action`]s to [`super::App`] state.

use anyhow::Result;

use crate::tui::screens::library_browse::LibrarySubsection;
use romm_api::core::library_scan::ScanCacheInvalidate;

use super::background::types::{RomLoadDone, RomLoadEvent};
use super::event::Action;
use super::App;
use super::AppScreen;

impl App {
    /// Apply background-only actions synchronously (tests and legacy poll path).
    pub(in crate::tui::app) fn apply_background(&mut self, action: Action) {
        match action {
            Action::LibraryMetadataRefreshComplete(done) => {
                self.apply_library_metadata_refresh(done);
            }
            Action::RomLoadComplete(done) => self.apply_rom_load_complete(done),
            Action::CollectionPrefetchComplete(done) => {
                self.apply_collection_prefetch_complete(done);
            }
            Action::SearchLoadComplete(done) => self.apply_search_load_complete(done),
            Action::CoverLoadComplete(done) => self.apply_cover_load_complete(done),
            Action::SaveListComplete(done) => self.apply_save_list_complete(done),
            Action::SaveUploadComplete(done) => self.apply_save_upload_complete(done),
            Action::SaveDownloadComplete(done) => self.apply_save_download_complete(done),
            Action::DeviceListComplete(done) => self.apply_device_list_complete(done),
            Action::PlatformListComplete(done) => self.apply_platform_list_complete(done),
            Action::SyncPushPullComplete(done) => self.apply_sync_push_pull_complete(done),
            Action::LibraryUploadProgress { uploaded, total } => {
                self.apply_library_upload_progress(uploaded, total);
            }
            Action::LibraryUploadComplete(result) => self.apply_library_upload_complete(result),
            Action::LibraryScanComplete(result) => self.apply_library_scan_complete(result),
            Action::DriveCollectionPrefetch => self.drive_collection_prefetch_scheduler(),
            Action::PollLibraryFooterClear => {
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    lib.poll_footer_clear();
                }
            }
            _ => {}
        }
    }

    /// Returns `true` when the loop should exit ([`Action::Quit`]).
    pub(crate) async fn update(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Quit => return Ok(true),
            Action::DismissGlobalMessage => {
                self.global_error = None;
                self.global_notice = None;
            }
            Action::DismissStartupSplash => {
                self.startup_splash = None;
            }
            Action::ShowKeyboardHelp => self.show_keyboard_help = true,
            Action::HideKeyboardHelp => self.show_keyboard_help = false,
            Action::ToggleDownloadOverlay => self.toggle_download_screen(),
            Action::CloseDownloadOverlay => {
                if matches!(self.screen, AppScreen::Download(_)) {
                    let stored = self.screen_before_download.take();
                    self.restore_screen_or_library(stored);
                }
            }
            Action::ToggleSearchOverlay => self.toggle_search_screen(),
            Action::ToggleSettingsOverlay => self.toggle_settings_screen(),
            Action::RescanLibrary(inv) => self.apply_rescan_library(inv),
            Action::ToggleLibraryUploadPrompt => self.apply_toggle_library_upload_prompt(),
            Action::ProcessDeferredRomLoad => self.process_deferred_rom_load(),
            Action::ApplyStartupUpdate => self.apply_startup_update().await,
            Action::StartupUpdatePromptStart => {
                if let Some(ref mut prompt) = self.startup_update_prompt {
                    if !prompt.updating {
                        prompt.updating = true;
                    }
                }
            }
            Action::StartupUpdatePromptOpenChangelog => {
                if let Some(ref prompt) = self.startup_update_prompt {
                    if let Err(err) = crate::update::open_changelog_in_browser() {
                        self.global_error = Some(format!("Could not open changelog: {err:#}"));
                    } else {
                        self.global_notice =
                            Some(format!("Opened changelog: {}", prompt.status.changelog_url));
                    }
                }
            }
            Action::StartupUpdatePromptDismiss => {
                self.startup_update_prompt = None;
            }
            Action::LibraryKey(key) => {
                if self.handle_library_browse(&key).await? {
                    return Ok(true);
                }
            }
            Action::SearchKey(key) => {
                if self.handle_search(&key).await? {
                    return Ok(true);
                }
            }
            Action::SettingsKey(key) => {
                if self.handle_settings(&key).await? {
                    return Ok(true);
                }
            }
            Action::GameDetailKey(key) => {
                if self.handle_game_detail(&key)? {
                    return Ok(true);
                }
            }
            Action::ExtrasPickerKey(key) => {
                if self.handle_extras_picker(&key)? {
                    return Ok(true);
                }
            }
            Action::SetupWizardKey(key) => {
                if self.handle_setup_wizard(&key).await? {
                    return Ok(true);
                }
            }
            action @ Action::LibraryMetadataRefreshComplete(_)
            | action @ Action::RomLoadComplete(_)
            | action @ Action::CollectionPrefetchComplete(_)
            | action @ Action::SearchLoadComplete(_)
            | action @ Action::CoverLoadComplete(_)
            | action @ Action::SaveListComplete(_)
            | action @ Action::SaveUploadComplete(_)
            | action @ Action::SaveDownloadComplete(_)
            | action @ Action::DeviceListComplete(_)
            | action @ Action::PlatformListComplete(_)
            | action @ Action::SyncPushPullComplete(_)
            | action @ Action::LibraryUploadProgress { .. }
            | action @ Action::LibraryUploadComplete(_)
            | action @ Action::LibraryScanComplete(_)
            | action @ Action::DriveCollectionPrefetch
            | action @ Action::PollLibraryFooterClear => {
                self.apply_background(action);
            }
        }
        Ok(false)
    }

    fn apply_rescan_library(&mut self, inv: ScanCacheInvalidate) {
        if let AppScreen::LibraryBrowse(ref lib) = self.screen {
            if !lib.any_upload_prompt_open()
                && !self.library_upload_inflight
                && !self.library_scan_inflight
            {
                self.spawn_library_rescan_worker(inv);
            }
        }
    }

    fn apply_toggle_library_upload_prompt(&mut self) {
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            if lib.any_upload_prompt_open() {
                lib.close_upload_prompt();
            } else if !self.library_upload_inflight && !self.library_scan_inflight {
                if lib.subsection == LibrarySubsection::ByConsole {
                    lib.open_upload_prompt();
                } else {
                    lib.set_metadata_footer(Some("Upload requires Consoles view — press t".into()));
                }
            }
        }
    }

    async fn apply_startup_update(&mut self) {
        let Some(ref mut prompt) = self.startup_update_prompt else {
            return;
        };
        if !prompt.updating {
            return;
        }

        if prompt.status.latest_version == "9.9.9-mock" {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            self.global_notice = Some("Mock update successful! (No files were changed)".into());
            self.startup_update_prompt = None;
            return;
        }

        let options = crate::update::ApplyUpdateOptions {
            show_progress: false,
            show_output: false,
            no_confirm: true,
            target_version_tag: Some(prompt.status.release_tag.clone()),
        };
        match crate::update::apply_update(None, options).await {
            Ok(crate::update::ApplyUpdateOutcome::Updated(version)) => {
                self.global_notice = Some(format!(
                    "Updated to {version}. Restart romm-cli to use the new version."
                ));
            }
            Ok(crate::update::ApplyUpdateOutcome::UpToDate(version)) => {
                self.global_notice = Some(format!("Already up to date (`{version}`)."));
            }
            Err(err) => {
                self.global_error = Some(format!("Update failed: {err:#}"));
            }
        }
        self.startup_update_prompt = None;
    }

    fn process_deferred_rom_load(&mut self) {
        let Some((key, req, expected, context, started)) = self.deferred_load_roms.take() else {
            return;
        };

        if let Some(ref k) = key {
            if let Some(cached) = self.rom_cache.get_valid(k, expected) {
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    if super::rom_load::primary_rom_load_result_matches_selection(lib, &key) {
                        lib.set_roms(cached.clone());
                        lib.set_rom_loading(false);
                        tracing::debug!(
                            "rom-list-render context={} latency_ms={} (cache_hit)",
                            context,
                            started.elapsed().as_millis()
                        );
                    } else {
                        lib.set_rom_loading(false);
                        tracing::debug!(
                            "rom-list-render context={} skipped stale cache hit",
                            context
                        );
                    }
                }
                return;
            }
        }

        if started.elapsed() < std::time::Duration::from_millis(250) {
            self.deferred_load_roms = Some((key, req, expected, context, started));
            return;
        }

        let gen = self.rom_load_gen;
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            lib.set_rom_loading(expected > 0);
        }
        if expected == 0 {
            if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                lib.set_rom_loading(false);
            }
            return;
        }

        let Some(r) = req else {
            if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                lib.set_rom_loading(false);
            }
            return;
        };
        let client = self.client.clone();
        let tx = self.rom_load_tx.clone();

        self.rom_load_task = Some(tokio::spawn(async move {
            let mut req = r;
            let mut aggregated: Option<romm_api::types::RomList> = None;

            loop {
                match client.call(&req).await {
                    Ok(mut batch) => {
                        if let Some(ref mut all) = aggregated {
                            if batch.items.is_empty() {
                                break;
                            }
                            all.items.append(&mut batch.items);
                            let _ = tx.send(RomLoadDone {
                                gen,
                                key: key.clone(),
                                expected,
                                event: RomLoadEvent::Batch(all.clone()),
                                context,
                                started,
                            });
                            if all.items.len() as u64 >= all.total {
                                break;
                            }
                            req.offset = Some(all.items.len() as u32);
                        } else {
                            let loaded = batch.items.len() as u64;
                            let total = batch.total;
                            let _ = tx.send(RomLoadDone {
                                gen,
                                key: key.clone(),
                                expected,
                                event: RomLoadEvent::Batch(batch.clone()),
                                context,
                                started,
                            });
                            req.offset = Some(loaded as u32);
                            aggregated = Some(batch);
                            if loaded >= total {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(RomLoadDone {
                            gen,
                            key: key.clone(),
                            expected,
                            event: RomLoadEvent::Failed(format!("{e:#}")),
                            context,
                            started,
                        });
                        return;
                    }
                }
                if let Some(ref all) = aggregated {
                    if all.items.len() >= 20000 {
                        break;
                    }
                }
            }

            let _ = tx.send(RomLoadDone {
                gen,
                key,
                expected,
                event: RomLoadEvent::Complete,
                context,
                started,
            });
        }));
    }

    fn apply_rom_load_complete(&mut self, done: RomLoadDone) {
        if !super::rom_load::primary_rom_load_result_is_current(done.gen, self.rom_load_gen) {
            return;
        }
        let AppScreen::LibraryBrowse(ref mut lib) = self.screen else {
            return;
        };
        if !super::rom_load::primary_rom_load_result_matches_selection(lib, &done.key) {
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

    fn apply_collection_prefetch_complete(
        &mut self,
        done: super::background::types::CollectionPrefetchDone,
    ) {
        self.collection_prefetch_inflight_keys.remove(&done.key);
        if let Some(roms) = done.roms {
            self.rom_cache.insert(done.key, roms, done.expected);
        } else if let Some(warning) = done.warning {
            tracing::debug!("{warning}");
        }
    }

    fn apply_search_load_complete(&mut self, done: super::background::types::SearchLoadDone) {
        use super::background::types::SearchLoadEvent;
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

    fn apply_cover_load_complete(&mut self, done: super::background::types::CoverLoadDone) {
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

    fn apply_save_list_complete(&mut self, done: super::background::types::SaveListDone) {
        if let AppScreen::GameDetail(detail) = &mut self.screen {
            if detail.rom.id == done.rom_id {
                match done.result {
                    Ok(rows) => detail.apply_saves(rows),
                    Err(e) => detail.apply_saves_error(e),
                }
            }
        }
    }

    fn apply_save_upload_complete(&mut self, done: super::background::types::SaveUploadDone) {
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

    fn apply_save_download_complete(&mut self, done: super::background::types::SaveDownloadDone) {
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

    fn apply_device_list_complete(&mut self, done: super::background::types::DeviceListDone) {
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

    fn apply_platform_list_complete(&mut self, done: super::background::types::PlatformListDone) {
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

    fn apply_sync_push_pull_complete(&mut self, done: super::background::types::SyncPushPullDone) {
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
        result: Result<super::background::types::LibraryUploadComplete, String>,
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
