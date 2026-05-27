//! Background task spawn and poll helpers for [`super::App`].

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::commands::library_scan::ScanCacheInvalidate;
use crate::core::cache::RomCacheKey;
use crate::core::startup_library_snapshot;
use crate::types::SaveMetadata;
use ratatui::style::Color;

use super::super::AppScreen;
use super::types::{
    CoverLoadDone, LibraryMetadataRefreshDone, LibraryUploadComplete, RomLoadEvent, SaveListDone,
    SearchLoadEvent,
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

    /// Drain background work (e.g. library metadata refresh). Safe to call each frame.
    pub fn poll_background_tasks(&mut self) {
        self.poll_library_metadata_refresh();
        self.poll_rom_load_results();
        self.poll_collection_prefetch_results();
        self.poll_search_load_results();
        self.poll_cover_load_results();
        self.poll_save_results();
        self.poll_settings_results();
        self.poll_library_upload();
        self.poll_library_scan();
        self.drive_collection_prefetch_scheduler();
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            lib.poll_footer_clear();
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
                let start =
                    crate::commands::library_scan::start_scan_library(&client, None).await?;
                crate::commands::library_scan::wait_for_task_terminal(
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
            .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    fn poll_library_scan(&mut self) {
        let Some(rx) = &mut self.library_scan_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(result) => {
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
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                self.library_scan_rx = None;
                self.library_scan_inflight = false;
                self.library_scan_pending_invalidate = None;
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
            let result: Result<LibraryUploadComplete, String> = async {
                client
                    .upload_rom(platform_id, &path, move |uploaded, total| {
                        let _ = prog_tx.send((uploaded, total));
                    })
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(LibraryUploadComplete {
                    platform_id,
                    scan_after,
                })
            }
            .await;
            let _ = done_tx.send(result);
        });
    }

    fn poll_library_upload(&mut self) {
        if let Some(rx) = &mut self.library_upload_progress_rx {
            while let Ok((up, tot)) = rx.try_recv() {
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    lib.set_metadata_footer(Some(format!(
                        "Uploading {} / {}…",
                        Self::format_upload_bytes(up),
                        Self::format_upload_bytes(tot)
                    )));
                }
            }
        }

        let Some(rx) = &mut self.library_upload_done_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(result) => {
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
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                self.library_upload_done_rx = None;
                self.library_upload_progress_rx = None;
                self.library_upload_inflight = false;
            }
        }
    }

    fn poll_search_load_results(&mut self) {
        loop {
            match self.search_load_rx.try_recv() {
                Ok(done) => {
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
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
    }

    pub(in crate::tui::app) fn spawn_cover_load_worker(&mut self, rom_id: u64, url: String) {
        if let Some(task) = self.cover_load_task.take() {
            task.abort();
        }
        let tx = self.cover_load_tx.clone();
        self.cover_load_task = Some(tokio::spawn(async move {
            let result = async {
                let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
                let status = response.status();
                if !status.is_success() {
                    return Err(format!("HTTP {}", status.as_u16()));
                }
                let bytes = response.bytes().await.map_err(|e| e.to_string())?;
                image::load_from_memory(&bytes).map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(CoverLoadDone { rom_id, result });
        }));
    }

    fn poll_cover_load_results(&mut self) {
        loop {
            match self.cover_load_rx.try_recv() {
                Ok(done) => {
                    if let AppScreen::GameDetail(detail) = &mut self.screen {
                        if detail.rom.id != done.rom_id {
                            continue;
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
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
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
                SaveMetadata::from_api_value(value)
            }
            .await
            .map_err(|e| format!("{e:#}"));
            let _ = tx.send(SaveListDone { rom_id, result });
        });
    }

    pub(in crate::tui::app) fn refresh_current_game_saves(&mut self) {
        if let AppScreen::GameDetail(detail) = &self.screen {
            self.spawn_save_list_worker(detail.rom.id);
        }
    }

    fn poll_save_results(&mut self) {
        while let Ok(done) = self.save_list_rx.try_recv() {
            if let AppScreen::GameDetail(detail) = &mut self.screen {
                if detail.rom.id == done.rom_id {
                    match done.result {
                        Ok(rows) => detail.apply_saves(rows),
                        Err(e) => detail.apply_saves_error(e),
                    }
                }
            }
        }
        while let Ok(done) = self.save_upload_rx.try_recv() {
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
        while let Ok(done) = self.save_download_rx.try_recv() {
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
    }

    fn poll_settings_results(&mut self) {
        while let Ok(done) = self.device_list_rx.try_recv() {
            if let AppScreen::Settings(settings) = &mut self.screen {
                match done.result {
                    Ok(devices) => {
                        settings.set_devices(devices);
                        settings.message = None;
                    }
                    Err(e) => {
                        settings.set_device_error(e.clone());
                        settings.message = Some((format!("Device load failed: {e}"), Color::Red));
                    }
                }
            }
        }
        while let Ok(done) = self.platform_list_rx.try_recv() {
            if let AppScreen::Settings(settings) = &mut self.screen {
                match done.result {
                    Ok(platforms) => {
                        settings.set_console_platforms(platforms);
                        settings.message = None;
                    }
                    Err(e) => {
                        settings.set_console_platform_error(e.clone());
                        settings.message = Some((format!("Platform load failed: {e}"), Color::Red));
                    }
                }
            }
        }
        while let Ok(done) = self.sync_push_pull_rx.try_recv() {
            if let AppScreen::Settings(settings) = &mut self.screen {
                settings.sync_inflight = false;
                match done.result {
                    Ok(session) => {
                        settings.message = Some((
                            format!("Sync session #{}: {}", session.id, session.status),
                            Color::Green,
                        ));
                    }
                    Err(e) => {
                        settings.message = Some((format!("Sync failed: {e}"), Color::Red));
                    }
                }
            }
        }
    }

    fn poll_rom_load_results(&mut self) {
        loop {
            match self.rom_load_rx.try_recv() {
                Ok(done) => {
                    if !crate::tui::app::rom_load::primary_rom_load_result_is_current(
                        done.gen,
                        self.rom_load_gen,
                    ) {
                        continue;
                    }
                    let AppScreen::LibraryBrowse(ref mut lib) = self.screen else {
                        continue;
                    };
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
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
    }

    fn poll_library_metadata_refresh(&mut self) {
        let mut batch = Vec::new();
        let mut disconnected = false;
        if let Some(rx) = &mut self.library_metadata_rx {
            loop {
                match rx.try_recv() {
                    Ok(msg) => batch.push(msg),
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
        for msg in batch {
            self.apply_library_metadata_refresh(msg);
        }
    }

    pub(in crate::tui::app) fn apply_library_metadata_refresh(
        &mut self,
        msg: LibraryMetadataRefreshDone,
    ) {
        if msg.gen != self.library_metadata_refresh_gen {
            return;
        }
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

        let old_digest =
            startup_library_snapshot::build_collection_digest_from_collections(&lib.collections);
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

        if selection_changed && lib.list_len() > 0 {
            lib.clear_roms();
            let key = lib.cache_key();
            let expected = lib.expected_rom_count();
            let req = Self::selected_rom_request_for_library(lib);
            lib.set_rom_loading(expected > 0);
            self.deferred_load_roms =
                Some((key, req, expected, "refresh_selection", Instant::now()));
        }

        let force_reload = std::mem::take(&mut self.force_rom_reload_after_metadata);
        if force_reload && lib.list_len() > 0 && !selection_changed {
            lib.clear_roms();
            let key = lib.cache_key();
            let expected = lib.expected_rom_count();
            let req = Self::selected_rom_request_for_library(lib);
            lib.set_rom_loading(expected > 0);
            self.deferred_load_roms =
                Some((key, req, expected, "post_scan_reload", Instant::now()));
        }

        self.queue_collection_prefetches_from_screen(1, "refresh_warmup");
    }

    fn poll_collection_prefetch_results(&mut self) {
        loop {
            match self.collection_prefetch_rx.try_recv() {
                Ok(done) => {
                    self.collection_prefetch_inflight_keys.remove(&done.key);
                    if let Some(roms) = done.roms {
                        self.rom_cache.insert(done.key, roms, done.expected);
                    } else if let Some(warning) = done.warning {
                        tracing::debug!("{warning}");
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
    }
}
