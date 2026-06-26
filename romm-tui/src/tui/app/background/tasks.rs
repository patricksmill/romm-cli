//! Background task spawn and poll helpers for [`super::App`].

use std::path::PathBuf;
use std::time::Duration;

use romm_api::core::cache::RomCacheKey;
use romm_api::core::library_scan::ScanCacheInvalidate;
use romm_api::core::metadata::{invalidate_platform_rom_cache, search_metadata_matches};
use romm_api::core::startup_library_snapshot;
use romm_api::endpoints::roms::{GetRom, PutRom};
use romm_api::error::{from_anyhow, ApiError, RommError};
use romm_api::types::SaveMetadata;

use super::super::event::{AppEvent, BackgroundAction};
use super::super::AppScreen;
use super::types::{
    CoverLoadDone, LibraryMetadataRefreshDone, LibraryUploadComplete, MetadataApplyDone,
    MetadataSearchDone, SaveListDone,
};

impl super::super::App {
    pub(in crate::tui::app) fn spawn_library_metadata_refresh(&mut self) {
        self.library_metadata_refresh_gen = self.library_metadata_refresh_gen.saturating_add(1);
        let gen = self.library_metadata_refresh_gen;
        let client = self.client.clone();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.library_metadata_rx = Some(rx);
        tokio::spawn(async move {
            let fetch = startup_library_snapshot::fetch_merged_library_metadata(&client).await;
            let _ = tx.send(LibraryMetadataRefreshDone {
                gen,
                platforms: fetch.platforms,
                collections: fetch.collections,
                collection_digest: fetch.collection_digest,
                warnings: fetch.warnings,
            });
        });
    }

    /// Drain background channels into [`AppEvent`]s. Safe to call each frame.
    pub(crate) fn drain_background_events(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        events.extend(self.drain_library_metadata_refresh());
        events.extend(self.drain_rom_load_results());
        events.extend(self.drain_collection_prefetch_results());
        events.extend(self.drain_search_load_results());
        events.extend(self.drain_cover_load_results());
        events.extend(self.drain_save_results());
        events.extend(self.drain_metadata_results());
        events.extend(self.drain_settings_results());
        events.extend(self.drain_library_upload());
        events.extend(self.drain_library_scan());
        self.drive_collection_prefetch_scheduler();
        events.push(AppEvent::Background(BackgroundAction::DrivePrefetch));
        events.push(AppEvent::Background(BackgroundAction::PollFooterClear));
        events
    }

    /// Legacy test entry: drain background events and apply synchronously.
    pub fn poll_background_tasks(&mut self) {
        let events = self.drain_background_events();
        for event in events {
            if let AppEvent::Background(bg) = event {
                self.apply_background(bg);
            }
        }
    }

    pub(in crate::tui::app) fn spawn_library_rescan_worker(
        &mut self,
        cache_on_success: ScanCacheInvalidate,
    ) {
        if self.library_scan_inflight {
            return;
        }
        self.library_scan_inflight = true;
        self.library_scan_pending_invalidate = Some(cache_on_success);
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            lib.set_metadata_footer(Some("Server library scan running…".into()));
        }
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.library_scan_rx = Some(rx);
        let client = self.client.clone();
        tokio::spawn(async move {
            let result = async {
                let start = romm_api::core::library_scan::start_scan_library(&client, None).await?;
                romm_api::core::library_scan::wait_for_task_terminal(
                    &client,
                    &start.task_id,
                    Duration::from_secs(3600),
                    None,
                    |_| {},
                )
                .await?;
                Ok::<(), anyhow::Error>(())
            }
            .await
            .map_err(from_anyhow);
            let _ = tx.send(result);
        });
    }

    fn drain_library_scan(&mut self) -> Vec<AppEvent> {
        let Some(rx) = &mut self.library_scan_rx else {
            return Vec::new();
        };
        match rx.try_recv() {
            Ok(result) => {
                vec![AppEvent::Background(BackgroundAction::LibraryScanDone(
                    result,
                ))]
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Vec::new(),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                self.library_scan_rx = None;
                self.library_scan_inflight = false;
                self.library_scan_pending_invalidate = None;
                Vec::new()
            }
        }
    }

    pub(in crate::tui::app) fn apply_library_scan_cache_invalidate(
        &mut self,
        inv: &ScanCacheInvalidate,
    ) {
        match inv {
            ScanCacheInvalidate::None => {}
            ScanCacheInvalidate::Platform(pid) => {
                self.rom_cache.remove(&RomCacheKey::Platform(*pid));
            }
            ScanCacheInvalidate::AllPlatforms => {
                self.rom_cache.remove_all_platform_entries();
                if let AppScreen::LibraryBrowse(lib) = &self.screen {
                    if let Some(ref k) = lib.cache_key() {
                        if !matches!(k, RomCacheKey::Platform(_)) {
                            self.rom_cache.remove(k);
                        }
                    }
                }
            }
        }
    }

    pub(in crate::tui::app) fn on_library_scan_completed_success(&mut self) {
        let inv = self
            .library_scan_pending_invalidate
            .take()
            .unwrap_or(ScanCacheInvalidate::AllPlatforms);
        self.apply_library_scan_cache_invalidate(&inv);
        if matches!(self.screen, AppScreen::LibraryBrowse(_)) {
            self.force_rom_reload_after_metadata = true;
            self.spawn_library_metadata_refresh();
        }
    }

    pub(in crate::tui::app) fn format_upload_bytes(n: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        if n >= GB {
            format!("{:.2} GiB", n as f64 / GB as f64)
        } else if n >= MB {
            format!("{:.2} MiB", n as f64 / MB as f64)
        } else if n >= KB {
            format!("{:.1} KiB", n as f64 / KB as f64)
        } else {
            format!("{n} B")
        }
    }

    pub(in crate::tui::app) fn spawn_library_upload_worker(
        &mut self,
        platform_id: u64,
        path: PathBuf,
        scan_after: bool,
    ) {
        if self.library_upload_inflight || self.library_scan_inflight {
            return;
        }
        self.library_upload_inflight = true;
        self.library_upload_progress_rx = None;
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            lib.set_metadata_footer(Some("Preparing upload…".into()));
        }
        let (prog_tx, prog_rx) = tokio::sync::mpsc::unbounded_channel();
        let (done_tx, done_rx) = tokio::sync::mpsc::unbounded_channel();
        self.library_upload_progress_rx = Some(prog_rx);
        self.library_upload_done_rx = Some(done_rx);
        let client = self.client.clone();
        tokio::spawn(async move {
            let result: Result<LibraryUploadComplete, RommError> = async {
                client
                    .upload_rom(platform_id, &path, move |uploaded, total| {
                        let _ = prog_tx.send((uploaded, total));
                    })
                    .await?;
                Ok(LibraryUploadComplete {
                    platform_id,
                    scan_after,
                })
            }
            .await;
            let _ = done_tx.send(result);
        });
    }

    fn drain_library_upload(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        if let Some(rx) = &mut self.library_upload_progress_rx {
            while let Ok((up, tot)) = rx.try_recv() {
                events.push(AppEvent::Background(
                    BackgroundAction::LibraryUploadProgress {
                        uploaded: up,
                        total: tot,
                    },
                ));
            }
        }

        let Some(rx) = &mut self.library_upload_done_rx else {
            return events;
        };
        match rx.try_recv() {
            Ok(result) => {
                events.push(AppEvent::Background(BackgroundAction::LibraryUploadDone(
                    result,
                )));
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                self.library_upload_done_rx = None;
                self.library_upload_progress_rx = None;
                self.library_upload_inflight = false;
            }
        }
        events
    }

    fn drain_search_load_results(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        loop {
            match self.search_load_rx.try_recv() {
                Ok(done) => events.push(AppEvent::Background(BackgroundAction::SearchLoad(done))),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        events
    }

    pub(in crate::tui::app) fn spawn_cover_load_worker(&mut self, rom_id: u64, url: String) {
        if let Some(task) = self.cover_load_task.take() {
            task.abort();
        }
        let tx = self.cover_load_tx.clone();
        self.cover_load_task = Some(tokio::spawn(async move {
            let result = async {
                let response = reqwest::get(&url)
                    .await
                    .map_err(|e| RommError::Api(ApiError::from(e)))?;
                let status = response.status();
                if !status.is_success() {
                    return Err(RommError::Api(ApiError::from_http_response(
                        status,
                        format!("cover HTTP {}", status.as_u16()),
                    )));
                }
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| RommError::Api(ApiError::from(e)))?;
                image::load_from_memory(&bytes)
                    .map_err(|e| RommError::Other(format!("invalid cover image: {e}")))
            }
            .await;
            let _ = tx.send(CoverLoadDone { rom_id, result });
        }));
    }

    fn drain_cover_load_results(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        loop {
            match self.cover_load_rx.try_recv() {
                Ok(done) => events.push(AppEvent::Background(BackgroundAction::CoverLoad(done))),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        events
    }

    pub(in crate::tui::app) fn maybe_start_game_detail_cover_load(&mut self) {
        let (rom_id, url) = match &mut self.screen {
            AppScreen::GameDetail(detail) => {
                if !detail.should_request_cover_load() {
                    return;
                }
                detail.set_cover_loading();
                let Some(url) = detail.cover_last_url.clone() else {
                    return;
                };
                (detail.rom.id, url)
            }
            _ => return,
        };
        self.spawn_cover_load_worker(rom_id, url);
    }

    pub(in crate::tui::app) fn spawn_save_list_worker(&mut self, rom_id: u64) {
        if let AppScreen::GameDetail(detail) = &mut self.screen {
            detail.set_saves_loading();
        }
        let client = self.client.clone();
        let tx = self.save_list_tx.clone();
        tokio::spawn(async move {
            let result = async {
                let value = client
                    .request_json(
                        "GET",
                        "/api/saves",
                        &[("rom_id".to_string(), rom_id.to_string())],
                        None,
                    )
                    .await?;
                SaveMetadata::from_api_value(value).map_err(from_anyhow)
            }
            .await;
            let _ = tx.send(SaveListDone { rom_id, result });
        });
    }

    pub(in crate::tui::app) fn refresh_current_game_saves(&mut self) {
        if let AppScreen::GameDetail(detail) = &self.screen {
            self.spawn_save_list_worker(detail.rom.id);
        }
    }

    pub(in crate::tui::app) fn spawn_metadata_search_worker(
        &mut self,
        rom_id: u64,
        search_term: String,
    ) {
        let client = self.client.clone();
        let tx = self.metadata_search_tx.clone();
        tokio::spawn(async move {
            let result =
                search_metadata_matches(&client, rom_id, Some(search_term), Some("name".into()))
                    .await
                    .map_err(RommError::from);
            let _ = tx.send(MetadataSearchDone { rom_id, result });
        });
    }

    pub(in crate::tui::app) fn spawn_metadata_apply_worker(
        &mut self,
        rom_id: u64,
        platform_id: u64,
        fields: romm_api::endpoints::roms::RomUpdateFields,
        unmatch_metadata: bool,
    ) {
        let client = self.client.clone();
        let tx = self.metadata_apply_tx.clone();
        tokio::spawn(async move {
            let result: Result<Box<romm_api::types::Rom>, RommError> = async {
                let ep = if unmatch_metadata {
                    PutRom::unmatch(rom_id)
                } else {
                    PutRom {
                        rom_id,
                        fields,
                        remove_cover: false,
                        unmatch_metadata: false,
                        artwork: None,
                    }
                };
                client.update_rom(&ep).await?;
                invalidate_platform_rom_cache(platform_id);
                client
                    .call(&GetRom { id: rom_id })
                    .await
                    .map(Box::new)
                    .map_err(RommError::from)
            }
            .await;
            let _ = tx.send(MetadataApplyDone {
                rom_id,
                platform_id,
                result,
            });
        });
    }

    fn drain_metadata_results(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        while let Ok(done) = self.metadata_search_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::MetadataSearch(done)));
        }
        while let Ok(done) = self.metadata_apply_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::MetadataApply(done)));
        }
        events
    }

    fn drain_save_results(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        while let Ok(done) = self.save_list_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::SaveList(done)));
        }
        while let Ok(done) = self.save_upload_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::SaveUpload(done)));
        }
        while let Ok(done) = self.save_download_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::SaveDownload(done)));
        }
        events
    }

    fn drain_settings_results(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        while let Ok(done) = self.device_list_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::DeviceList(done)));
        }
        while let Ok(done) = self.platform_list_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::PlatformList(done)));
        }
        while let Ok(done) = self.sync_push_pull_rx.try_recv() {
            events.push(AppEvent::Background(BackgroundAction::SyncPushPull(done)));
        }
        events
    }

    fn drain_rom_load_results(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        loop {
            match self.rom_load_rx.try_recv() {
                Ok(done) => events.push(AppEvent::Background(BackgroundAction::RomLoad(done))),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        events
    }

    fn drain_library_metadata_refresh(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let mut disconnected = false;
        if let Some(rx) = &mut self.library_metadata_rx {
            loop {
                match rx.try_recv() {
                    Ok(msg) => events.push(AppEvent::Background(
                        BackgroundAction::LibraryMetadataRefresh(msg),
                    )),
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        if disconnected {
            self.library_metadata_rx = None;
        }
        events
    }

    pub(in crate::tui::app) fn apply_library_metadata_refresh(
        &mut self,
        msg: LibraryMetadataRefreshDone,
    ) {
        if msg.gen != self.library_metadata_refresh_gen {
            return;
        }

        let (selection_reload, post_scan_reload) = {
            let AppScreen::LibraryBrowse(ref mut lib) = self.screen else {
                return;
            };

            let had_cached_lists = !lib.platforms.is_empty() || !lib.collections.is_empty();
            let live_empty = msg.collections.is_empty();
            if live_empty && had_cached_lists && !msg.warnings.is_empty() {
                lib.set_temporary_metadata_footer(
                    "Could not refresh library metadata (keeping cached list).".into(),
                    std::time::Duration::from_secs(3),
                );
                self.force_rom_reload_after_metadata = false;
                return;
            }

            let old_digest = startup_library_snapshot::build_collection_digest_from_collections(
                &lib.collections,
            );
            let digest_changed = old_digest != msg.collection_digest;
            let update_platforms = !msg.platforms.is_empty();
            let selection_changed = lib.replace_metadata_preserving_selection(
                msg.platforms,
                msg.collections,
                update_platforms,
                true,
            );
            startup_library_snapshot::save_snapshot(&lib.platforms, &lib.collections);

            let footer = if msg.warnings.is_empty() {
                if digest_changed {
                    Some("Collection metadata updated.".into())
                } else {
                    None
                }
            } else {
                let w = msg.warnings.join(" | ");
                let short: String = if w.chars().count() > 160 {
                    let prefix: String = w.chars().take(157).collect();
                    format!("{prefix}…")
                } else {
                    w
                };
                Some(format!("Partial refresh: {}", short))
            };
            lib.set_metadata_footer(footer);

            let force_reload = std::mem::take(&mut self.force_rom_reload_after_metadata);
            let selection_reload = if selection_changed && lib.list_len() > 0 {
                lib.clear_roms();
                let key = lib.cache_key();
                let expected = lib.expected_rom_count();
                let req = Self::selected_rom_request_for_library(lib);
                lib.set_rom_loading(expected > 0);
                Some((key, req, expected, "refresh_selection"))
            } else {
                None
            };
            let post_scan_reload = if force_reload && lib.list_len() > 0 && !selection_changed {
                lib.clear_roms();
                let key = lib.cache_key();
                let expected = lib.expected_rom_count();
                let req = Self::selected_rom_request_for_library(lib);
                lib.set_rom_loading(expected > 0);
                Some((key, req, expected, "post_scan_reload"))
            } else {
                None
            };
            (selection_reload, post_scan_reload)
        };

        if let Some((key, req, expected, context)) = selection_reload {
            self.queue_primary_rom_load(key, req, expected, context);
        } else if let Some((key, req, expected, context)) = post_scan_reload {
            self.queue_primary_rom_load(key, req, expected, context);
        }

        self.queue_collection_prefetches_from_screen(1, "refresh_warmup");
    }

    fn drain_collection_prefetch_results(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        loop {
            match self.collection_prefetch_rx.try_recv() {
                Ok(done) => events.push(AppEvent::Background(
                    BackgroundAction::CollectionPrefetch(done),
                )),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        events
    }
}
