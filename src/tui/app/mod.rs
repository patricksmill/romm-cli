//! Application state and TUI event loop.
//!
//! Submodules:
//! - [`background`] — async task spawn/poll
//! - [`handlers`] — per-screen key handlers
//! - [`run`] — terminal event loop
//! - [`render`] — frame drawing
//! - [`rom_load`] — ROM fetch and prefetch scheduling

mod background;
mod handlers;
mod render;
mod rom_load;
mod run;

#[cfg(test)]
mod tests;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::{HashSet, VecDeque};

use crate::client::RommClient;
use crate::commands::library_scan::ScanCacheInvalidate;
use crate::config::Config;
use crate::core::cache::{RomCache, RomCacheKey};
use crate::core::download::DownloadManager;
use crate::endpoints::roms::GetRoms;
use crate::feature_compat::SaveSyncCompatibility;
use crate::update::UpdateStatus;

use super::screens::connected_splash::StartupSplash;
use super::screens::{
    DownloadScreen, ExtrasPickerScreen, GameDetailScreen, LibraryBrowseScreen, MainMenuScreen,
    SearchScreen, SettingsScreen,
};
use super::theme::resolve_theme_or_default;
use ratatui_themekit::Theme;

use background::types::{
    CollectionPrefetchDone, CoverLoadDone, DeferredLoadRoms, DeviceListDone,
    LibraryMetadataRefreshDone, LibraryUploadComplete, PlatformListDone, RomLoadDone,
    SaveDownloadDone, SaveListDone, SaveUploadDone, SearchLoadDone, StartupUpdatePrompt,
    SyncPushPullDone,
};

/// All possible high-level screens in the TUI.
///
/// `App` holds exactly one of these at a time and delegates both
/// rendering and key handling based on the current variant.
pub enum AppScreen {
    MainMenu(MainMenuScreen),
    LibraryBrowse(Box<LibraryBrowseScreen>),
    Search(SearchScreen),
    Settings(Box<SettingsScreen>),
    GameDetail(Box<GameDetailScreen>),
    ExtrasPicker(Box<ExtrasPickerScreen>),
    Download(DownloadScreen),
    SetupWizard(Box<crate::tui::screens::setup_wizard::SetupWizard>),
}

/// Root application object for the TUI.
///
/// Owns shared services (`RommClient`, `RomCache`, `DownloadManager`)
/// as well as the currently active [`AppScreen`].
pub struct App {
    pub screen: AppScreen,
    client: RommClient,
    config: Config,
    /// RomM server version from `GET /api/heartbeat` (`SYSTEM.VERSION`), if available.
    server_version: Option<String>,
    save_sync_compat: SaveSyncCompatibility,
    rom_cache: RomCache,
    downloads: DownloadManager,
    /// Screen to restore when closing the Download overlay.
    screen_before_download: Option<AppScreen>,
    /// Deferred ROM load: (cache_key, api_request, expected_rom_count, context, start).
    deferred_load_roms: Option<DeferredLoadRoms>,
    /// Brief “connected” banner after setup or when the server responds to heartbeat.
    startup_splash: Option<StartupSplash>,
    pub global_error: Option<String>,
    pub global_notice: Option<String>,
    show_keyboard_help: bool,
    startup_update_prompt: Option<StartupUpdatePrompt>,
    /// Receives completed background metadata refreshes for the library screen.
    library_metadata_rx: Option<tokio::sync::mpsc::UnboundedReceiver<LibraryMetadataRefreshDone>>,
    /// Incremented each time a new refresh is spawned; stale completions are ignored.
    library_metadata_refresh_gen: u64,
    collection_prefetch_rx: tokio::sync::mpsc::UnboundedReceiver<CollectionPrefetchDone>,
    collection_prefetch_tx: tokio::sync::mpsc::UnboundedSender<CollectionPrefetchDone>,
    collection_prefetch_queue: VecDeque<(RomCacheKey, GetRoms, u64)>,
    collection_prefetch_queued_keys: HashSet<RomCacheKey>,
    collection_prefetch_inflight_keys: HashSet<RomCacheKey>,
    /// Latest generation for primary ROM loads; completions with a lower gen are ignored.
    rom_load_gen: u64,
    rom_load_rx: tokio::sync::mpsc::UnboundedReceiver<RomLoadDone>,
    rom_load_tx: tokio::sync::mpsc::UnboundedSender<RomLoadDone>,
    rom_load_task: Option<tokio::task::JoinHandle<()>>,
    search_load_rx: tokio::sync::mpsc::UnboundedReceiver<SearchLoadDone>,
    search_load_tx: tokio::sync::mpsc::UnboundedSender<SearchLoadDone>,
    search_load_task: Option<tokio::task::JoinHandle<()>>,
    cover_load_rx: tokio::sync::mpsc::UnboundedReceiver<CoverLoadDone>,
    cover_load_tx: tokio::sync::mpsc::UnboundedSender<CoverLoadDone>,
    cover_load_task: Option<tokio::task::JoinHandle<()>>,
    /// Receives `Ok(())` when a background `scan_library` (with wait) finishes successfully.
    library_scan_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Result<(), String>>>,
    library_scan_inflight: bool,
    /// Cache policy applied when the current background scan completes successfully.
    library_scan_pending_invalidate: Option<ScanCacheInvalidate>,
    /// After a successful server scan, force ROM list reload once metadata refresh completes.
    force_rom_reload_after_metadata: bool,
    /// Background chunked ROM upload to the selected platform.
    library_upload_inflight: bool,
    library_upload_progress_rx: Option<tokio::sync::mpsc::UnboundedReceiver<(u64, u64)>>,
    library_upload_done_rx:
        Option<tokio::sync::mpsc::UnboundedReceiver<Result<LibraryUploadComplete, String>>>,
    save_list_rx: tokio::sync::mpsc::UnboundedReceiver<SaveListDone>,
    save_list_tx: tokio::sync::mpsc::UnboundedSender<SaveListDone>,
    save_upload_rx: tokio::sync::mpsc::UnboundedReceiver<SaveUploadDone>,
    save_upload_tx: tokio::sync::mpsc::UnboundedSender<SaveUploadDone>,
    save_download_rx: tokio::sync::mpsc::UnboundedReceiver<SaveDownloadDone>,
    save_download_tx: tokio::sync::mpsc::UnboundedSender<SaveDownloadDone>,
    device_list_rx: tokio::sync::mpsc::UnboundedReceiver<DeviceListDone>,
    device_list_tx: tokio::sync::mpsc::UnboundedSender<DeviceListDone>,
    platform_list_rx: tokio::sync::mpsc::UnboundedReceiver<PlatformListDone>,
    platform_list_tx: tokio::sync::mpsc::UnboundedSender<PlatformListDone>,
    sync_push_pull_rx: tokio::sync::mpsc::UnboundedReceiver<SyncPushPullDone>,
    sync_push_pull_tx: tokio::sync::mpsc::UnboundedSender<SyncPushPullDone>,
    theme: Box<dyn Theme>,
}

impl App {
    fn blocks_global_chord_shortcuts(&self) -> bool {
        self.startup_splash.is_some()
            || self.startup_update_prompt.is_some()
            || self.global_error.is_some()
            || self.global_notice.is_some()
    }

    fn blocks_global_d_shortcut(&self) -> bool {
        let base = match &self.screen {
            AppScreen::Search(_) | AppScreen::Settings(_) | AppScreen::SetupWizard(_) => true,
            AppScreen::LibraryBrowse(lib) => {
                lib.any_search_bar_open() || lib.any_upload_prompt_open()
            }
            _ => false,
        };
        base || self.library_upload_inflight || self.blocks_global_chord_shortcuts()
    }

    fn allows_global_question_help(&self) -> bool {
        match &self.screen {
            AppScreen::Search(_) | AppScreen::SetupWizard(_) => false,
            AppScreen::LibraryBrowse(lib)
                if lib.any_search_bar_open() || lib.any_upload_prompt_open() =>
            {
                false
            }
            AppScreen::Settings(s) if s.editing || s.path_picker.is_some() => false,
            _ => true,
        }
    }

    pub(crate) fn is_force_quit_key(key: &crossterm::event::KeyEvent) -> bool {
        key.kind == KeyEventKind::Press
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    }
    /// Construct a new `App` with fresh cache and empty download list.
    pub fn new(
        client: RommClient,
        config: Config,
        save_sync_compat: SaveSyncCompatibility,
        server_version: Option<String>,
        startup_splash: Option<StartupSplash>,
        startup_update: Option<UpdateStatus>,
    ) -> Self {
        let (prefetch_tx, prefetch_rx) = tokio::sync::mpsc::unbounded_channel();
        let (rom_load_tx, rom_load_rx) = tokio::sync::mpsc::unbounded_channel();
        let (search_load_tx, search_load_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cover_load_tx, cover_load_rx) = tokio::sync::mpsc::unbounded_channel();
        let (save_list_tx, save_list_rx) = tokio::sync::mpsc::unbounded_channel();
        let (save_upload_tx, save_upload_rx) = tokio::sync::mpsc::unbounded_channel();
        let (save_download_tx, save_download_rx) = tokio::sync::mpsc::unbounded_channel();
        let (device_list_tx, device_list_rx) = tokio::sync::mpsc::unbounded_channel();
        let (platform_list_tx, platform_list_rx) = tokio::sync::mpsc::unbounded_channel();
        let (sync_push_pull_tx, sync_push_pull_rx) = tokio::sync::mpsc::unbounded_channel();
        let theme = resolve_theme_or_default(&config.theme);
        Self {
            screen: AppScreen::MainMenu(MainMenuScreen::new()),
            client,
            config,
            server_version,
            save_sync_compat,
            rom_cache: RomCache::load(),
            downloads: DownloadManager::new(),
            screen_before_download: None,
            deferred_load_roms: None,
            startup_splash,
            global_error: None,
            global_notice: None,
            show_keyboard_help: false,
            startup_update_prompt: startup_update.map(|status| StartupUpdatePrompt {
                status,
                updating: false,
            }),
            library_metadata_rx: None,
            library_metadata_refresh_gen: 0,
            collection_prefetch_rx: prefetch_rx,
            collection_prefetch_tx: prefetch_tx,
            collection_prefetch_queue: VecDeque::new(),
            collection_prefetch_queued_keys: HashSet::new(),
            collection_prefetch_inflight_keys: HashSet::new(),
            rom_load_gen: 0,
            rom_load_rx,
            rom_load_tx,
            rom_load_task: None,
            search_load_rx,
            search_load_tx,
            search_load_task: None,
            cover_load_rx,
            cover_load_tx,
            cover_load_task: None,
            library_scan_rx: None,
            library_scan_inflight: false,
            library_scan_pending_invalidate: None,
            force_rom_reload_after_metadata: false,
            library_upload_inflight: false,
            library_upload_progress_rx: None,
            library_upload_done_rx: None,
            save_list_rx,
            save_list_tx,
            save_upload_rx,
            save_upload_tx,
            save_download_rx,
            save_download_tx,
            device_list_rx,
            device_list_tx,
            platform_list_rx,
            platform_list_tx,
            sync_push_pull_rx,
            sync_push_pull_tx,
            theme,
        }
    }
    pub fn set_error(&mut self, err: anyhow::Error) {
        self.global_error = Some(format!("{:#}", err));
    }
    pub async fn handle_key_event(&mut self, key: &KeyEvent) -> Result<bool> {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        if self.global_error.is_some() || self.global_notice.is_some() {
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                self.global_error = None;
                self.global_notice = None;
            }
            return Ok(false);
        }

        // Dismiss the connected splash before the update prompt: the splash is drawn on top,
        // so Enter/Esc here must not be routed to the hidden update dialog (which would quit).
        if self.startup_splash.is_some() {
            self.startup_splash = None;
            return Ok(false);
        }

        if self.startup_update_prompt.is_some() {
            return self.handle_startup_update_prompt(key).await;
        }

        if self.show_keyboard_help {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Enter | KeyCode::F(1) | KeyCode::Char('?')
            ) {
                self.show_keyboard_help = false;
            }
            return Ok(false);
        }

        if key.code == KeyCode::F(1) {
            self.show_keyboard_help = true;
            return Ok(false);
        }
        if key.code == KeyCode::Char('?') && self.allows_global_question_help() {
            self.show_keyboard_help = true;
            return Ok(false);
        }

        // Global shortcut: 'd' toggles Download overlay (not on screens that need free typing / menus).
        if key.code == KeyCode::Char('d') && !self.blocks_global_d_shortcut() {
            self.toggle_download_screen();
            return Ok(false);
        }

        match &self.screen {
            AppScreen::MainMenu(_) => self.handle_main_menu(key).await,
            AppScreen::LibraryBrowse(_) => self.handle_library_browse(key).await,
            AppScreen::Search(_) => self.handle_search(key).await,
            AppScreen::Settings(_) => self.handle_settings(key).await,
            AppScreen::GameDetail(_) => self.handle_game_detail(key),
            AppScreen::ExtrasPicker(_) => self.handle_extras_picker(key),
            AppScreen::Download(_) => self.handle_download(key),
            AppScreen::SetupWizard(_) => self.handle_setup_wizard(key).await,
        }
    }
}
