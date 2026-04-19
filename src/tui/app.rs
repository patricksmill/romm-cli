//! Application state and TUI event loop.
//!
//! The `App` struct owns long-lived state (config, HTTP client, cache,
//! downloads, and the currently active `AppScreen`). It drives a simple
//! state machine:
//! - render the current screen,
//! - wait for input,
//! - dispatch the key to a small handler per screen.
//!
//! This is intentionally separated from the drawing code in `screens/`
//! so that alternative frontends can reuse the same \"backend\" services.

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::style::Color;
use ratatui::Terminal;
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use crate::client::RommClient;
use crate::config::{auth_for_persist_merge, normalize_romm_origin, Config};
use crate::core::cache::{RomCache, RomCacheKey};
use crate::core::download::DownloadManager;
use crate::core::startup_library_snapshot;
use crate::endpoints::roms::GetRoms;
use crate::types::{Collection, RomList};

use super::keyboard_help;
use super::openapi::{resolve_path_template, EndpointRegistry};
use super::screens::connected_splash::{self, StartupSplash};
use super::screens::setup_wizard::SetupWizard;
use super::screens::{
    BrowseScreen, DownloadScreen, ExecuteScreen, GameDetailPrevious, GameDetailScreen,
    LibraryBrowseScreen, MainMenuScreen, ResultDetailScreen, ResultScreen, SearchScreen,
    SettingsScreen,
};

/// Result of a background library metadata refresh (generation-guarded).
struct LibraryMetadataRefreshDone {
    gen: u64,
    collections: Vec<Collection>,
    collection_digest: Vec<startup_library_snapshot::CollectionDigestEntry>,
    warnings: Vec<String>,
}

struct CollectionPrefetchDone {
    key: RomCacheKey,
    expected: u64,
    roms: Option<RomList>,
    warning: Option<String>,
}

/// Background primary ROM list fetch (deferred load path). Generation-guarded against stale completions.
struct RomLoadDone {
    gen: u64,
    key: Option<RomCacheKey>,
    expected: u64,
    result: Result<RomList, String>,
    context: &'static str,
    started: Instant,
}

struct SearchLoadDone {
    result: Result<RomList, String>,
}

/// Deferred primary ROM load: cache key, API request, expected count, context label, start time.
type DeferredLoadRoms = (
    Option<RomCacheKey>,
    Option<GetRoms>,
    u64,
    &'static str,
    Instant,
);

#[inline]
fn primary_rom_load_result_is_current(done_gen: u64, current_gen: u64) -> bool {
    done_gen == current_gen
}

// ---------------------------------------------------------------------------
// Screen enum
// ---------------------------------------------------------------------------

/// All possible high-level screens in the TUI.
///
/// `App` holds exactly one of these at a time and delegates both
/// rendering and key handling based on the current variant.
pub enum AppScreen {
    MainMenu(MainMenuScreen),
    LibraryBrowse(LibraryBrowseScreen),
    Search(SearchScreen),
    Settings(SettingsScreen),
    Browse(BrowseScreen),
    Execute(ExecuteScreen),
    Result(ResultScreen),
    ResultDetail(ResultDetailScreen),
    GameDetail(Box<GameDetailScreen>),
    Download(DownloadScreen),
    SetupWizard(Box<crate::tui::screens::setup_wizard::SetupWizard>),
}

fn blocks_global_d_shortcut(screen: &AppScreen) -> bool {
    match screen {
        AppScreen::Search(_) | AppScreen::Settings(_) | AppScreen::SetupWizard(_) => true,
        AppScreen::LibraryBrowse(lib) => lib.any_search_bar_open(),
        _ => false,
    }
}

fn allows_global_question_help(screen: &AppScreen) -> bool {
    match screen {
        AppScreen::Search(_) | AppScreen::SetupWizard(_) | AppScreen::Execute(_) => false,
        AppScreen::LibraryBrowse(lib) if lib.any_search_bar_open() => false,
        AppScreen::Settings(s) if s.editing => false,
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

/// Root application object for the TUI.
///
/// Owns shared services (`RommClient`, `RomCache`, `DownloadManager`)
/// as well as the currently active [`AppScreen`].
pub struct App {
    pub screen: AppScreen,
    client: RommClient,
    config: Config,
    registry: EndpointRegistry,
    /// RomM server version from `GET /api/heartbeat` (`SYSTEM.VERSION`), if available.
    server_version: Option<String>,
    rom_cache: RomCache,
    downloads: DownloadManager,
    /// Screen to restore when closing the Download overlay.
    screen_before_download: Option<AppScreen>,
    /// Deferred ROM load: (cache_key, api_request, expected_rom_count, context, start).
    deferred_load_roms: Option<DeferredLoadRoms>,
    /// Brief “connected” banner after setup or when the server responds to heartbeat.
    startup_splash: Option<StartupSplash>,
    pub global_error: Option<String>,
    show_keyboard_help: bool,
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
}

impl App {
    fn is_force_quit_key(key: &crossterm::event::KeyEvent) -> bool {
        key.kind == KeyEventKind::Press
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    }

    fn selected_rom_request_for_library(
        lib: &super::screens::library_browse::LibraryBrowseScreen,
    ) -> Option<GetRoms> {
        match lib.subsection {
            super::screens::library_browse::LibrarySubsection::ByConsole => {
                lib.get_roms_request_platform()
            }
            super::screens::library_browse::LibrarySubsection::ByCollection => {
                lib.get_roms_request_collection()
            }
        }
    }

    /// Construct a new `App` with fresh cache and empty download list.
    pub fn new(
        client: RommClient,
        config: Config,
        registry: EndpointRegistry,
        server_version: Option<String>,
        startup_splash: Option<StartupSplash>,
    ) -> Self {
        let (prefetch_tx, prefetch_rx) = tokio::sync::mpsc::unbounded_channel();
        let (rom_load_tx, rom_load_rx) = tokio::sync::mpsc::unbounded_channel();
        let (search_load_tx, search_load_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            screen: AppScreen::MainMenu(MainMenuScreen::new()),
            client,
            config,
            registry,
            server_version,
            rom_cache: RomCache::load(),
            downloads: DownloadManager::new(),
            screen_before_download: None,
            deferred_load_roms: None,
            startup_splash,
            global_error: None,
            show_keyboard_help: false,
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
        }
    }

    fn spawn_library_metadata_refresh(&mut self) {
        self.library_metadata_refresh_gen = self.library_metadata_refresh_gen.saturating_add(1);
        let gen = self.library_metadata_refresh_gen;
        let client = self.client.clone();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.library_metadata_rx = Some(rx);
        tokio::spawn(async move {
            let fetch = startup_library_snapshot::fetch_collection_summaries(&client).await;
            let _ = tx.send(LibraryMetadataRefreshDone {
                gen,
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
        self.drive_collection_prefetch_scheduler();
    }

    fn poll_search_load_results(&mut self) {
        loop {
            match self.search_load_rx.try_recv() {
                Ok(done) => {
                    if let AppScreen::Search(ref mut search) = self.screen {
                        search.loading = false;
                        if let Ok(roms) = done.result {
                            search.set_results(roms);
                        }
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
    }

    fn poll_rom_load_results(&mut self) {
        loop {
            match self.rom_load_rx.try_recv() {
                Ok(done) => {
                    if !primary_rom_load_result_is_current(done.gen, self.rom_load_gen) {
                        continue;
                    }
                    let AppScreen::LibraryBrowse(ref mut lib) = self.screen else {
                        continue;
                    };
                    match done.result {
                        Ok(roms) => {
                            if let Some(ref k) = done.key {
                                self.rom_cache
                                    .insert(k.clone(), roms.clone(), done.expected);
                            }
                            lib.set_roms(roms);
                            tracing::debug!(
                                "rom-list-render context={} latency_ms={}",
                                done.context,
                                done.started.elapsed().as_millis()
                            );
                        }
                        Err(e) => {
                            lib.set_metadata_footer(Some(format!("Could not load games: {e}")));
                        }
                    }
                    lib.set_rom_loading(false);
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

    fn apply_library_metadata_refresh(&mut self, msg: LibraryMetadataRefreshDone) {
        if msg.gen != self.library_metadata_refresh_gen {
            return;
        }
        let AppScreen::LibraryBrowse(ref mut lib) = self.screen else {
            return;
        };

        let had_cached_lists = !lib.platforms.is_empty() || !lib.collections.is_empty();
        let live_empty = msg.collections.is_empty();
        if live_empty && had_cached_lists && !msg.warnings.is_empty() {
            lib.set_metadata_footer(Some(
                "Could not refresh library metadata (keeping cached list).".into(),
            ));
            return;
        }

        let old_digest =
            startup_library_snapshot::build_collection_digest_from_collections(&lib.collections);
        let digest_changed = old_digest != msg.collection_digest;
        let selection_changed =
            lib.replace_metadata_preserving_selection(Vec::new(), msg.collections, false, true);
        startup_library_snapshot::save_snapshot(&lib.platforms, &lib.collections);

        let footer = if msg.warnings.is_empty() {
            if digest_changed {
                Some("Collection metadata updated.".into())
            } else {
                Some("Collection metadata already up to date.".into())
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
        self.queue_collection_prefetches_from_screen(1, "refresh_warmup");
    }

    fn queue_collection_prefetches_from_screen(&mut self, radius: usize, _reason: &'static str) {
        let AppScreen::LibraryBrowse(ref lib) = self.screen else {
            return;
        };
        for (key, req, expected) in lib.collection_prefetch_candidates(radius) {
            if self.rom_cache.get_valid(&key, expected).is_some() {
                continue;
            }
            if self.collection_prefetch_queued_keys.contains(&key)
                || self.collection_prefetch_inflight_keys.contains(&key)
            {
                continue;
            }
            self.collection_prefetch_queued_keys.insert(key.clone());
            self.collection_prefetch_queue
                .push_back((key, req, expected));
        }
    }

    fn drive_collection_prefetch_scheduler(&mut self) {
        const PREFETCH_MAX_INFLIGHT: usize = 2;
        while self.collection_prefetch_inflight_keys.len() < PREFETCH_MAX_INFLIGHT {
            let Some((key, req, expected)) = self.collection_prefetch_queue.pop_back() else {
                break;
            };
            self.collection_prefetch_queued_keys.remove(&key);
            self.collection_prefetch_inflight_keys.insert(key.clone());
            let tx = self.collection_prefetch_tx.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                let result = Self::fetch_roms_full(client, req).await;
                let (roms, warning) = match result {
                    Ok(list) => (Some(list), None),
                    Err(e) => (None, Some(format!("Collection prefetch failed: {e:#}"))),
                };
                let _ = tx.send(CollectionPrefetchDone {
                    key,
                    expected,
                    roms,
                    warning,
                });
            });
        }
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

    pub fn set_error(&mut self, err: anyhow::Error) {
        self.global_error = Some(format!("{:#}", err));
    }

    // -----------------------------------------------------------------------
    // Event loop
    // -----------------------------------------------------------------------

    /// Main TUI event loop.
    ///
    /// This method owns the terminal for the lifetime of the app,
    /// repeatedly drawing the current screen and dispatching key
    /// events until the user chooses to quit.
    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            self.poll_background_tasks();
            if self
                .startup_splash
                .as_ref()
                .is_some_and(|s| s.should_auto_dismiss())
            {
                self.startup_splash = None;
            }
            // Draw the current screen. `App::render` delegates to the
            // appropriate screen type based on `self.screen`.
            terminal.draw(|f| self.render(f))?;

            // Poll with a short timeout so the UI refreshes during downloads
            // even when the user is not pressing any keys.
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if Self::is_force_quit_key(&key) {
                        break;
                    }
                    if key.kind == KeyEventKind::Press && self.handle_key(key.code).await? {
                        break;
                    }
                }
            }

            // Process deferred ROM fetch (set during LibraryBrowse ↑/↓, subsection switch, refresh).
            // Cache hits apply synchronously; network fetch runs in a background task so the loop
            // never awaits HTTP and the UI stays responsive (see `poll_rom_load_results`).
            if let Some((key, req, expected, context, started)) = self.deferred_load_roms.take() {
                // Fast path: valid disk cache — no await, no spawn, load immediately.
                if let Some(ref k) = key {
                    if let Some(cached) = self.rom_cache.get_valid(k, expected) {
                        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                            lib.set_roms(cached.clone());
                            lib.set_rom_loading(false);
                            tracing::debug!(
                                "rom-list-render context={} latency_ms={} (cache_hit)",
                                context,
                                started.elapsed().as_millis()
                            );
                        }
                        continue;
                    }
                }

                // Debounce network fetches
                if started.elapsed() < std::time::Duration::from_millis(250) {
                    // Put it back to keep waiting
                    self.deferred_load_roms = Some((key, req, expected, context, started));
                    continue;
                }

                self.rom_load_gen = self.rom_load_gen.saturating_add(1);
                let gen = self.rom_load_gen;
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    lib.set_rom_loading(expected > 0);
                }
                if expected == 0 {
                    if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                        lib.set_rom_loading(false);
                    }
                    continue;
                }
                
                let Some(r) = req else {
                    if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                        lib.set_rom_loading(false);
                    }
                    continue;
                };
                let client = self.client.clone();
                let tx = self.rom_load_tx.clone();
                
                if let Some(task) = self.rom_load_task.take() {
                    task.abort();
                }
                
                self.rom_load_task = Some(tokio::spawn(async move {
                    let result = Self::fetch_roms_full(client, r)
                        .await
                        .map_err(|e| format!("{e:#}"));
                    let _ = tx.send(RomLoadDone {
                        gen,
                        key,
                        expected,
                        result,
                        context,
                        started,
                    });
                }));
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // ROM fetch (used by background tasks and collection prefetch)
    // -----------------------------------------------------------------------

    async fn fetch_roms_full(client: RommClient, req: GetRoms) -> Result<RomList> {
        let mut roms = client.call(&req).await?;
        let total = roms.total;
        let ceiling = 20000;
        while (roms.items.len() as u64) < total && (roms.items.len() as u64) < ceiling {
            let mut next_req = req.clone();
            next_req.offset = Some(roms.items.len() as u32);
            let next_batch = client.call(&next_req).await?;
            if next_batch.items.is_empty() {
                break;
            }
            roms.items.extend(next_batch.items);
        }
        Ok(roms)
    }

    // -----------------------------------------------------------------------
    // Key dispatch — one small method per screen
    // -----------------------------------------------------------------------

    pub async fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        if self.global_error.is_some() {
            if key == KeyCode::Esc || key == KeyCode::Enter {
                self.global_error = None;
            }
            return Ok(false);
        }

        if self.startup_splash.is_some() {
            self.startup_splash = None;
            return Ok(false);
        }

        if self.show_keyboard_help {
            if matches!(
                key,
                KeyCode::Esc | KeyCode::Enter | KeyCode::F(1) | KeyCode::Char('?')
            ) {
                self.show_keyboard_help = false;
            }
            return Ok(false);
        }

        if key == KeyCode::F(1) {
            self.show_keyboard_help = true;
            return Ok(false);
        }
        if key == KeyCode::Char('?') && allows_global_question_help(&self.screen) {
            self.show_keyboard_help = true;
            return Ok(false);
        }

        // Global shortcut: 'd' toggles Download overlay (not on screens that need free typing / menus).
        if key == KeyCode::Char('d') && !blocks_global_d_shortcut(&self.screen) {
            self.toggle_download_screen();
            return Ok(false);
        }

        match &self.screen {
            AppScreen::MainMenu(_) => self.handle_main_menu(key).await,
            AppScreen::LibraryBrowse(_) => self.handle_library_browse(key).await,
            AppScreen::Search(_) => self.handle_search(key).await,
            AppScreen::Settings(_) => self.handle_settings(key).await,
            AppScreen::Browse(_) => self.handle_browse(key),
            AppScreen::Execute(_) => self.handle_execute(key).await,
            AppScreen::Result(_) => self.handle_result(key),
            AppScreen::ResultDetail(_) => self.handle_result_detail(key),
            AppScreen::GameDetail(_) => self.handle_game_detail(key),
            AppScreen::Download(_) => self.handle_download(key),
            AppScreen::SetupWizard(_) => self.handle_setup_wizard(key).await,
        }
    }

    // -- Download overlay ---------------------------------------------------

    fn toggle_download_screen(&mut self) {
        let current =
            std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
        match current {
            AppScreen::Download(_) => {
                self.screen = self
                    .screen_before_download
                    .take()
                    .unwrap_or_else(|| AppScreen::MainMenu(MainMenuScreen::new()));
            }
            other => {
                self.screen_before_download = Some(other);
                self.screen = AppScreen::Download(DownloadScreen::new(self.downloads.shared()));
            }
        }
    }

    fn handle_download(&mut self, key: KeyCode) -> Result<bool> {
        if key == KeyCode::Esc || key == KeyCode::Char('d') {
            self.screen = self
                .screen_before_download
                .take()
                .unwrap_or_else(|| AppScreen::MainMenu(MainMenuScreen::new()));
        }
        Ok(false)
    }

    // -- Main menu ----------------------------------------------------------

    async fn handle_main_menu(&mut self, key: KeyCode) -> Result<bool> {
        let menu = match &mut self.screen {
            AppScreen::MainMenu(m) => m,
            _ => return Ok(false),
        };
        match key {
            KeyCode::Up | KeyCode::Char('k') => menu.previous(),
            KeyCode::Down | KeyCode::Char('j') => menu.next(),
            KeyCode::Enter => match menu.selected {
                0 => {
                    let start = Instant::now();
                    let snap = startup_library_snapshot::load_snapshot();
                    let (platforms, collections, from_disk) = match snap {
                        Some(s) => (s.platforms, s.collections, true),
                        None => (Vec::new(), Vec::new(), false),
                    };
                    let mut lib = LibraryBrowseScreen::new(platforms, collections);
                    if from_disk && lib.list_len() > 0 {
                        lib.set_metadata_footer(Some(
                            "Refreshing library metadata in background…".into(),
                        ));
                    } else if lib.list_len() == 0 {
                        lib.set_metadata_footer(Some("Loading library metadata…".into()));
                    }
                    if lib.list_len() > 0 {
                        let key = lib.cache_key();
                        let expected = lib.expected_rom_count();
                        let req = Self::selected_rom_request_for_library(&lib);
                        lib.set_rom_loading(expected > 0);
                        self.deferred_load_roms = Some((
                            key,
                            req,
                            expected,
                            "startup_first_selection",
                            Instant::now(),
                        ));
                    }
                    self.screen = AppScreen::LibraryBrowse(lib);
                    self.spawn_library_metadata_refresh();
                    tracing::debug!(
                        "library-open latency_ms={} snapshot_hit={}",
                        start.elapsed().as_millis(),
                        from_disk
                    );
                }
                1 => self.screen = AppScreen::Search(SearchScreen::new()),
                2 => {
                    self.screen_before_download = Some(AppScreen::MainMenu(MainMenuScreen::new()));
                    self.screen = AppScreen::Download(DownloadScreen::new(self.downloads.shared()));
                }
                3 => {
                    self.screen = AppScreen::Settings(SettingsScreen::new(
                        &self.config,
                        self.server_version.as_deref(),
                    ))
                }
                4 => return Ok(true),
                _ => {}
            },
            KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    // -- Library browse -----------------------------------------------------

    async fn handle_library_browse(&mut self, key: KeyCode) -> Result<bool> {
        use super::screens::library_browse::{LibrarySearchMode, LibraryViewMode};

        let lib = match &mut self.screen {
            AppScreen::LibraryBrowse(l) => l,
            _ => return Ok(false),
        };

        // List pane: search typing bar
        if lib.view_mode == LibraryViewMode::List {
            if let Some(mode) = lib.list_search.mode {
                let old_key = lib.cache_key();
                match key {
                    KeyCode::Esc => lib.clear_list_search(),
                    KeyCode::Backspace => lib.delete_list_search_char(),
                    KeyCode::Char(c) => lib.add_list_search_char(c),
                    KeyCode::Tab if mode == LibrarySearchMode::Jump => lib.list_jump_match(true),
                    KeyCode::Enter => lib.commit_list_filter_bar(),
                    _ => {}
                }
                let new_key = lib.cache_key();
                if old_key != new_key && lib.list_len() > 0 {
                    lib.clear_roms();
                    let expected = lib.expected_rom_count();
                    if expected > 0 {
                        let req = Self::selected_rom_request_for_library(lib);
                        lib.set_rom_loading(true);
                        self.deferred_load_roms =
                            Some((new_key, req, expected, "search_filter", Instant::now()));
                    } else {
                        lib.set_rom_loading(false);
                        self.deferred_load_roms = None;
                    }
                }
                return Ok(false);
            }
        }

        // Games pane: search typing bar
        if lib.view_mode == LibraryViewMode::Roms {
            if let Some(mode) = lib.rom_search.mode {
                match key {
                    KeyCode::Esc => lib.clear_rom_search(),
                    KeyCode::Backspace => lib.delete_rom_search_char(),
                    KeyCode::Char(c) => lib.add_rom_search_char(c),
                    KeyCode::Tab if mode == LibrarySearchMode::Jump => lib.jump_rom_match(true),
                    KeyCode::Enter => lib.commit_rom_filter_bar(),
                    _ => {}
                }
                return Ok(false);
            }
        }

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if lib.view_mode == LibraryViewMode::List {
                    lib.list_previous();
                    if lib.list_len() > 0 {
                        lib.clear_roms(); // avoid showing previous console's games
                        let key = lib.cache_key();
                        let expected = lib.expected_rom_count();
                        if expected > 0 {
                            let req = Self::selected_rom_request_for_library(lib);
                            lib.set_rom_loading(true);
                            self.deferred_load_roms =
                                Some((key, req, expected, "list_move_up", Instant::now()));
                        } else {
                            lib.set_rom_loading(false);
                            self.deferred_load_roms = None;
                        }
                        if lib.subsection
                            == super::screens::library_browse::LibrarySubsection::ByCollection
                        {
                            tracing::debug!("collections-selection move=up expected={expected}");
                            self.queue_collection_prefetches_from_screen(1, "move_up");
                        }
                    }
                } else {
                    lib.rom_previous();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if lib.view_mode == LibraryViewMode::List {
                    lib.list_next();
                    if lib.list_len() > 0 {
                        lib.clear_roms(); // avoid showing previous console's games
                        let key = lib.cache_key();
                        let expected = lib.expected_rom_count();
                        if expected > 0 {
                            let req = Self::selected_rom_request_for_library(lib);
                            lib.set_rom_loading(true);
                            self.deferred_load_roms =
                                Some((key, req, expected, "list_move_down", Instant::now()));
                        } else {
                            lib.set_rom_loading(false);
                            self.deferred_load_roms = None;
                        }
                        if lib.subsection
                            == super::screens::library_browse::LibrarySubsection::ByCollection
                        {
                            tracing::debug!("collections-selection move=down expected={expected}");
                            self.queue_collection_prefetches_from_screen(1, "move_down");
                        }
                    }
                } else {
                    lib.rom_next();
                }
            }
            KeyCode::Left | KeyCode::Char('h') if lib.view_mode == LibraryViewMode::Roms => {
                lib.back_to_list();
            }
            KeyCode::Right | KeyCode::Char('l') => lib.switch_view(),
            KeyCode::Tab => {
                if lib.view_mode == LibraryViewMode::List {
                    lib.switch_view();
                } else {
                    lib.switch_view(); // Normal tab also switches panels
                }
            }
            KeyCode::Char('/') => match lib.view_mode {
                LibraryViewMode::List => lib.enter_list_search(LibrarySearchMode::Filter),
                LibraryViewMode::Roms => lib.enter_rom_search(LibrarySearchMode::Filter),
            },
            KeyCode::Char('f') => match lib.view_mode {
                LibraryViewMode::List => lib.enter_list_search(LibrarySearchMode::Jump),
                LibraryViewMode::Roms => lib.enter_rom_search(LibrarySearchMode::Jump),
            },
            KeyCode::Enter => {
                if lib.view_mode == LibraryViewMode::List {
                    lib.switch_view();
                } else if let Some((primary, others)) = lib.get_selected_group() {
                    let lib_screen = std::mem::replace(
                        &mut self.screen,
                        AppScreen::MainMenu(MainMenuScreen::new()),
                    );
                    if let AppScreen::LibraryBrowse(l) = lib_screen {
                        self.screen = AppScreen::GameDetail(Box::new(GameDetailScreen::new(
                            primary,
                            others,
                            GameDetailPrevious::Library(l),
                            self.downloads.shared(),
                        )));
                    }
                }
            }
            KeyCode::Char('t') => {
                lib.switch_subsection();
                // `switch_subsection` clears ROMs but does not queue a load; mirror list ↑/↓ so the
                // first row in the new subsection (index 0) gets ROMs without an extra keypress.
                if lib.view_mode == LibraryViewMode::List && lib.list_len() > 0 {
                    let key = lib.cache_key();
                    let expected = lib.expected_rom_count();
                    if expected > 0 {
                        let req = Self::selected_rom_request_for_library(lib);
                        lib.set_rom_loading(true);
                        self.deferred_load_roms =
                            Some((key, req, expected, "switch_subsection", Instant::now()));
                    } else {
                        lib.set_rom_loading(false);
                        self.deferred_load_roms = None;
                    }
                }
                if lib.subsection == super::screens::library_browse::LibrarySubsection::ByCollection
                {
                    tracing::debug!("collections-subsection entered");
                    self.queue_collection_prefetches_from_screen(1, "enter_collections");
                }
            }
            KeyCode::Esc => {
                if lib.view_mode == LibraryViewMode::Roms {
                    if lib.rom_search.filter_browsing {
                        lib.clear_rom_search();
                    } else {
                        lib.back_to_list();
                    }
                } else if lib.list_search.filter_browsing {
                    lib.clear_list_search();
                } else {
                    self.screen = AppScreen::MainMenu(MainMenuScreen::new());
                }
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    // -- Search -------------------------------------------------------------

    async fn handle_search(&mut self, key: KeyCode) -> Result<bool> {
        let search = match &mut self.screen {
            AppScreen::Search(s) => s,
            _ => return Ok(false),
        };
        match key {
            KeyCode::Backspace => search.delete_char(),
            KeyCode::Left => search.cursor_left(),
            KeyCode::Right => search.cursor_right(),
            KeyCode::Up => search.previous(),
            KeyCode::Down => search.next(),
            KeyCode::Char(c) => search.add_char(c),
            KeyCode::Enter => {
                if search.query.is_empty() {
                    // no-op (same as before: empty query does not search)
                } else if search.result_groups.is_some() && search.results_match_current_query() {
                    if let Some((primary, others)) = search.get_selected_group() {
                        let prev = std::mem::replace(
                            &mut self.screen,
                            AppScreen::MainMenu(MainMenuScreen::new()),
                        );
                        if let AppScreen::Search(s) = prev {
                            self.screen = AppScreen::GameDetail(Box::new(GameDetailScreen::new(
                                primary,
                                others,
                                GameDetailPrevious::Search(s),
                                self.downloads.shared(),
                            )));
                        }
                    }
                } else {
                    let req = GetRoms {
                        search_term: Some(search.query.clone()),
                        limit: Some(50),
                        ..Default::default()
                    };
                    search.loading = true;
                    if let Some(task) = self.search_load_task.take() {
                        task.abort();
                    }
                    let client = self.client.clone();
                    let tx = self.search_load_tx.clone();
                    self.search_load_task = Some(tokio::spawn(async move {
                        let result = client.call(&req).await.map_err(|e| format!("{e:#}"));
                        let _ = tx.send(SearchLoadDone { result });
                    }));
                }
            }
            KeyCode::Esc => {
                if search.results.is_some() {
                    search.clear_results();
                } else {
                    self.screen = AppScreen::MainMenu(MainMenuScreen::new());
                }
            }
            _ => {}
        }
        Ok(false)
    }

    // -- Settings -----------------------------------------------------------

    async fn refresh_settings_server_version(&mut self) -> Result<()> {
        let (base_url, download_dir, use_https, verbose, auth) = {
            let settings = match &self.screen {
                AppScreen::Settings(s) => s,
                _ => return Ok(()),
            };
            let mut base_url = normalize_romm_origin(settings.base_url.trim());
            if settings.use_https && base_url.starts_with("http://") {
                base_url = base_url.replace("http://", "https://");
            }
            if !settings.use_https && base_url.starts_with("https://") {
                base_url = base_url.replace("https://", "http://");
            }
            (
                base_url,
                settings.download_dir.clone(),
                settings.use_https,
                self.client.verbose(),
                self.config.auth.clone(),
            )
        };
        let cfg = Config {
            base_url,
            download_dir,
            use_https,
            auth,
        };
        let client = match RommClient::new(&cfg, verbose) {
            Ok(c) => c,
            Err(_) => {
                if let AppScreen::Settings(s) = &mut self.screen {
                    s.server_version = "unavailable (invalid URL or client error)".to_string();
                    self.server_version = None;
                }
                return Ok(());
            }
        };
        let ver = client.rom_server_version_from_heartbeat().await;
        if let AppScreen::Settings(s) = &mut self.screen {
            match ver {
                Some(v) => {
                    s.server_version = v.clone();
                    self.server_version = Some(v);
                }
                None => {
                    s.server_version = "unavailable (heartbeat failed)".to_string();
                    self.server_version = None;
                }
            }
        }
        Ok(())
    }

    async fn handle_settings(&mut self, key: KeyCode) -> Result<bool> {
        let settings = match &mut self.screen {
            AppScreen::Settings(s) => s,
            _ => return Ok(false),
        };

        if settings.editing {
            match key {
                KeyCode::Enter => {
                    let idx = settings.selected_index;
                    settings.save_edit();
                    if idx == 0 {
                        self.refresh_settings_server_version().await?;
                    }
                }
                KeyCode::Esc => settings.cancel_edit(),
                KeyCode::Backspace => settings.delete_char(),
                KeyCode::Left => settings.move_cursor_left(),
                KeyCode::Right => settings.move_cursor_right(),
                KeyCode::Char(c) => settings.add_char(c),
                _ => {}
            }
            return Ok(false);
        }

        match key {
            KeyCode::Up | KeyCode::Char('k') => settings.previous(),
            KeyCode::Down | KeyCode::Char('j') => settings.next(),
            KeyCode::Enter => {
                if settings.selected_index == 3 {
                    self.screen =
                        AppScreen::SetupWizard(Box::new(SetupWizard::new_auth_only(&self.config)));
                } else {
                    let toggle_https = settings.selected_index == 2;
                    settings.enter_edit();
                    if toggle_https {
                        self.refresh_settings_server_version().await?;
                    }
                }
            }
            KeyCode::Char('s' | 'S') => {
                // Save to disk (accept both cases; footer shows "S:")
                use crate::config::persist_user_config;
                let auth = auth_for_persist_merge(self.config.auth.clone());
                if let Err(e) = persist_user_config(
                    &settings.base_url,
                    &settings.download_dir,
                    settings.use_https,
                    auth,
                ) {
                    settings.message = Some((format!("Error saving: {e}"), Color::Red));
                } else {
                    settings.message = Some(("Saved to config.json".to_string(), Color::Green));
                    // Update app state
                    self.config.base_url = settings.base_url.clone();
                    self.config.download_dir = settings.download_dir.clone();
                    self.config.use_https = settings.use_https;
                    // Re-create client to pick up new base URL
                    if let Ok(new_client) = RommClient::new(&self.config, self.client.verbose()) {
                        self.client = new_client;
                    }
                }
            }
            KeyCode::Esc => self.screen = AppScreen::MainMenu(MainMenuScreen::new()),
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    // -- API Browse ---------------------------------------------------------

    fn handle_browse(&mut self, key: KeyCode) -> Result<bool> {
        use super::screens::browse::ViewMode;

        let browse = match &mut self.screen {
            AppScreen::Browse(b) => b,
            _ => return Ok(false),
        };
        match key {
            KeyCode::Up | KeyCode::Char('k') => browse.previous(),
            KeyCode::Down | KeyCode::Char('j') => browse.next(),
            KeyCode::Left | KeyCode::Char('h') if browse.view_mode == ViewMode::Endpoints => {
                browse.switch_view();
            }
            KeyCode::Right | KeyCode::Char('l') if browse.view_mode == ViewMode::Sections => {
                browse.switch_view();
            }
            KeyCode::Tab => browse.switch_view(),
            KeyCode::Enter => {
                if browse.view_mode == ViewMode::Endpoints {
                    if let Some(ep) = browse.get_selected_endpoint() {
                        self.screen = AppScreen::Execute(ExecuteScreen::new(ep.clone()));
                    }
                } else {
                    browse.switch_view();
                }
            }
            KeyCode::Esc => self.screen = AppScreen::MainMenu(MainMenuScreen::new()),
            _ => {}
        }
        Ok(false)
    }

    // -- Execute endpoint ---------------------------------------------------

    async fn handle_execute(&mut self, key: KeyCode) -> Result<bool> {
        let execute = match &mut self.screen {
            AppScreen::Execute(e) => e,
            _ => return Ok(false),
        };
        match key {
            KeyCode::Tab => execute.next_field(),
            KeyCode::BackTab => execute.previous_field(),
            KeyCode::Char(c) => execute.add_char_to_focused(c),
            KeyCode::Backspace => execute.delete_char_from_focused(),
            KeyCode::Enter => {
                let endpoint = execute.endpoint.clone();
                let query = execute.get_query_params();
                let body = if endpoint.has_body && !execute.body_text.is_empty() {
                    Some(serde_json::from_str(&execute.body_text)?)
                } else {
                    None
                };
                let resolved_path =
                    match resolve_path_template(&endpoint.path, &execute.get_path_params()) {
                        Ok(p) => p,
                        Err(e) => {
                            self.screen = AppScreen::Result(ResultScreen::new(
                                serde_json::json!({ "error": format!("{e}") }),
                                None,
                                None,
                            ));
                            return Ok(false);
                        }
                    };
                match self
                    .client
                    .request_json(&endpoint.method, &resolved_path, &query, body)
                    .await
                {
                    Ok(result) => {
                        self.screen = AppScreen::Result(ResultScreen::new(
                            result,
                            Some(&endpoint.method),
                            Some(resolved_path.as_str()),
                        ));
                    }
                    Err(e) => {
                        self.screen = AppScreen::Result(ResultScreen::new(
                            serde_json::json!({ "error": format!("{e}") }),
                            None,
                            None,
                        ));
                    }
                }
            }
            KeyCode::Esc => {
                self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
            }
            _ => {}
        }
        Ok(false)
    }

    // -- Result view --------------------------------------------------------

    fn handle_result(&mut self, key: KeyCode) -> Result<bool> {
        use super::screens::result::ResultViewMode;

        let result = match &mut self.screen {
            AppScreen::Result(r) => r,
            _ => return Ok(false),
        };
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if result.view_mode == ResultViewMode::Json {
                    result.scroll_up(1);
                } else {
                    result.table_previous();
                }
            }
            KeyCode::Down => {
                if result.view_mode == ResultViewMode::Json {
                    result.scroll_down(1);
                } else {
                    result.table_next();
                }
            }
            KeyCode::Char('j') if result.view_mode == ResultViewMode::Json => {
                result.scroll_down(1);
            }
            KeyCode::PageUp => {
                if result.view_mode == ResultViewMode::Table {
                    result.table_page_up();
                } else {
                    result.scroll_up(10);
                }
            }
            KeyCode::PageDown => {
                if result.view_mode == ResultViewMode::Table {
                    result.table_page_down();
                } else {
                    result.scroll_down(10);
                }
            }
            KeyCode::Char('t') if result.table_row_count > 0 => {
                result.switch_view_mode();
            }
            KeyCode::Enter
                if result.view_mode == ResultViewMode::Table && result.table_row_count > 0 =>
            {
                if let Some(item) = result.get_selected_item_value() {
                    let prev = std::mem::replace(
                        &mut self.screen,
                        AppScreen::MainMenu(MainMenuScreen::new()),
                    );
                    if let AppScreen::Result(rs) = prev {
                        self.screen = AppScreen::ResultDetail(ResultDetailScreen::new(rs, item));
                    }
                }
            }
            KeyCode::Esc => {
                result.clear_message();
                self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    // -- Result detail ------------------------------------------------------

    fn handle_result_detail(&mut self, key: KeyCode) -> Result<bool> {
        let detail = match &mut self.screen {
            AppScreen::ResultDetail(d) => d,
            _ => return Ok(false),
        };
        match key {
            KeyCode::Up | KeyCode::Char('k') => detail.scroll_up(1),
            KeyCode::Down | KeyCode::Char('j') => detail.scroll_down(1),
            KeyCode::PageUp => detail.scroll_up(10),
            KeyCode::PageDown => detail.scroll_down(10),
            KeyCode::Char('o') => detail.open_image_url(),
            KeyCode::Esc => {
                detail.clear_message();
                let prev =
                    std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
                if let AppScreen::ResultDetail(d) = prev {
                    self.screen = AppScreen::Result(d.parent);
                }
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    // -- Game detail --------------------------------------------------------

    fn handle_game_detail(&mut self, key: KeyCode) -> Result<bool> {
        let detail = match &mut self.screen {
            AppScreen::GameDetail(d) => d,
            _ => return Ok(false),
        };

        // Acknowledge download completion on any key press
        // (check if there's a completed/errored download for this ROM)
        if !detail.download_completion_acknowledged {
            if let Ok(list) = detail.downloads.lock() {
                let has_completed = list.iter().any(|j| {
                    j.rom_id == detail.rom.id
                        && matches!(
                            j.status,
                            crate::core::download::DownloadStatus::Done
                                | crate::core::download::DownloadStatus::Error(_)
                        )
                });
                let is_still_downloading = list.iter().any(|j| {
                    j.rom_id == detail.rom.id
                        && matches!(j.status, crate::core::download::DownloadStatus::Downloading)
                });
                // Only acknowledge if there's a completion and no active download
                if has_completed && !is_still_downloading {
                    detail.download_completion_acknowledged = true;
                }
            }
        }

        match key {
            // Only start a download once per detail view and avoid
            // stacking multiple concurrent downloads for the same ROM.
            KeyCode::Enter if !detail.has_started_download => {
                detail.has_started_download = true;
                self.downloads
                    .start_download(&detail.rom, self.client.clone());
            }
            KeyCode::Char('o') => detail.open_cover(),
            KeyCode::Char('m') => detail.toggle_technical(),
            KeyCode::Esc => {
                detail.clear_message();
                let prev =
                    std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
                if let AppScreen::GameDetail(g) = prev {
                    self.screen = match g.previous {
                        GameDetailPrevious::Library(l) => AppScreen::LibraryBrowse(l),
                        GameDetailPrevious::Search(s) => AppScreen::Search(s),
                    };
                }
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    // -- Setup Wizard -------------------------------------------------------

    async fn handle_setup_wizard(&mut self, key: KeyCode) -> Result<bool> {
        let wizard = match &mut self.screen {
            AppScreen::SetupWizard(w) => w,
            _ => return Ok(false),
        };

        // Create a dummy event to pass to handle_key
        let event = crossterm::event::KeyEvent::new(key, crossterm::event::KeyModifiers::empty());
        if wizard.handle_key(event)? {
            // Esc pressed
            self.screen = AppScreen::Settings(SettingsScreen::new(
                &self.config,
                self.server_version.as_deref(),
            ));
            return Ok(false);
        }

        if wizard.testing {
            let result = wizard.try_connect_and_persist(self.client.verbose()).await;
            wizard.testing = false;
            match result {
                Ok(cfg) => {
                    let auth_ok = cfg.auth.is_some();
                    self.config = cfg;
                    if let Ok(new_client) = RommClient::new(&self.config, self.client.verbose()) {
                        self.client = new_client;
                    }
                    let mut settings =
                        SettingsScreen::new(&self.config, self.server_version.as_deref());
                    if auth_ok {
                        settings.message = Some((
                            "Authentication updated successfully".to_string(),
                            Color::Green,
                        ));
                    } else {
                        settings.message = Some((
                            "Saved configuration but credentials could not be loaded from the OS keyring (see logs)."
                                .to_string(),
                            Color::Yellow,
                        ));
                    }
                    self.screen = AppScreen::Settings(settings);
                }
                Err(e) => {
                    wizard.error = Some(format!("{e:#}"));
                }
            }
        }
        Ok(false)
    }

    // -----------------------------------------------------------------------
    // Render
    // -----------------------------------------------------------------------

    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = f.size();
        if let Some(ref splash) = self.startup_splash {
            connected_splash::render(f, area, splash);
            return;
        }
        match &mut self.screen {
            AppScreen::MainMenu(menu) => menu.render(f, area),
            AppScreen::LibraryBrowse(lib) => lib.render(f, area),
            AppScreen::Search(search) => {
                search.render(f, area);
                if let Some((x, y)) = search.cursor_position(area) {
                    f.set_cursor(x, y);
                }
            }
            AppScreen::Settings(settings) => {
                settings.render(f, area);
                if let Some((x, y)) = settings.cursor_position(area) {
                    f.set_cursor(x, y);
                }
            }
            AppScreen::Browse(browse) => browse.render(f, area),
            AppScreen::Execute(execute) => {
                execute.render(f, area);
                if let Some((x, y)) = execute.cursor_position(area) {
                    f.set_cursor(x, y);
                }
            }
            AppScreen::Result(result) => result.render(f, area),
            AppScreen::ResultDetail(detail) => detail.render(f, area),
            AppScreen::GameDetail(detail) => detail.render(f, area),
            AppScreen::Download(d) => d.render(f, area),
            AppScreen::SetupWizard(wizard) => {
                wizard.render(f, area);
                if let Some((x, y)) = wizard.cursor_pos(area) {
                    f.set_cursor(x, y);
                }
            }
        }

        if self.show_keyboard_help {
            keyboard_help::render_keyboard_help(f, area);
        }

        if let Some(ref err) = self.global_error {
            let popup_area = ratatui::layout::Rect {
                x: area.width.saturating_sub(60) / 2,
                y: area.height.saturating_sub(10) / 2,
                width: 60.min(area.width),
                height: 10.min(area.height),
            };
            f.render_widget(ratatui::widgets::Clear, popup_area);
            let block = ratatui::widgets::Block::default()
                .title("Error")
                .borders(ratatui::widgets::Borders::ALL)
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red));
            let text = format!("{}\n\nPress Esc to dismiss", err);
            let paragraph = ratatui::widgets::Paragraph::new(text)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(paragraph, popup_area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tui::openapi::EndpointRegistry;
    use crate::tui::screens::library_browse::LibraryBrowseScreen;
    use crate::types::Platform;
    use crossterm::event::{KeyEvent, KeyModifiers};
    use serde_json::json;

    fn platform(id: u64, name: &str, rom_count: u64) -> Platform {
        serde_json::from_value(json!({
            "id": id,
            "slug": format!("p{id}"),
            "fs_slug": format!("p{id}"),
            "rom_count": rom_count,
            "name": name,
            "igdb_slug": null,
            "moby_slug": null,
            "hltb_slug": null,
            "custom_name": null,
            "igdb_id": null,
            "sgdb_id": null,
            "moby_id": null,
            "launchbox_id": null,
            "ss_id": null,
            "ra_id": null,
            "hasheous_id": null,
            "tgdb_id": null,
            "flashpoint_id": null,
            "category": null,
            "generation": null,
            "family_name": null,
            "family_slug": null,
            "url": null,
            "url_logo": null,
            "firmware": [],
            "aspect_ratio": null,
            "created_at": "",
            "updated_at": "",
            "fs_size_bytes": 0,
            "is_unidentified": false,
            "is_identified": true,
            "missing_from_fs": false,
            "display_name": null
        }))
        .expect("valid platform fixture")
    }

    fn app_with_library(platforms: Vec<Platform>) -> App {
        let config = Config {
            base_url: "http://127.0.0.1:9".into(),
            download_dir: "/tmp".into(),
            use_https: false,
            auth: None,
        };
        let client = RommClient::new(&config, false).expect("client");
        let mut app = App::new(client, config, EndpointRegistry::default(), None, None);
        app.screen = AppScreen::LibraryBrowse(LibraryBrowseScreen::new(platforms, vec![]));
        app
    }

    #[tokio::test]
    async fn list_move_to_zero_rom_selection_does_not_queue_deferred_load() {
        let mut app = app_with_library(vec![platform(1, "HasRoms", 5), platform(2, "Empty", 0)]);

        assert!(!app.handle_key(KeyCode::Down).await.expect("key handled"));
        assert!(
            app.deferred_load_roms.is_none(),
            "selection move to zero-rom platform should not queue deferred ROM load"
        );
    }

    #[test]
    fn ctrl_c_is_treated_as_force_quit() {
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(App::is_force_quit_key(&ctrl_c));

        let plain_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty());
        assert!(!App::is_force_quit_key(&plain_c));
    }

    #[test]
    fn primary_rom_load_stale_gen_is_ignored() {
        assert!(!super::primary_rom_load_result_is_current(1, 2));
        assert!(super::primary_rom_load_result_is_current(3, 3));
    }
}
