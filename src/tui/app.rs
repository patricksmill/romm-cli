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
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::time::Duration;

use crate::client::RommClient;
use crate::config::Config;
use crate::core::cache::{RomCache, RomCacheKey};
use crate::core::download::DownloadManager;
use crate::endpoints::{collections::ListCollections, platforms::ListPlatforms, roms::GetRoms};
use crate::types::RomList;

use super::openapi::{resolve_path_template, EndpointRegistry};
use super::screens::connected_splash::{self, StartupSplash};
use super::screens::{
    BrowseScreen, DownloadScreen, ExecuteScreen, GameDetailPrevious, GameDetailScreen,
    LibraryBrowseScreen, MainMenuScreen, ResultDetailScreen, ResultScreen, SearchScreen,
    SettingsScreen,
};

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
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

/// Root application object for the TUI.
///
/// Owns shared services (`RommClient`, `RomCache`, `DownloadManager`)
/// as well as the currently active [`AppScreen`].
pub struct App {
    screen: AppScreen,
    client: RommClient,
    config: Config,
    registry: EndpointRegistry,
    /// RomM server version from `GET /api/heartbeat` (`SYSTEM.VERSION`), if available.
    server_version: Option<String>,
    rom_cache: RomCache,
    downloads: DownloadManager,
    /// Screen to restore when closing the Download overlay.
    screen_before_download: Option<AppScreen>,
    /// Deferred ROM load: (cache_key, api_request, expected_rom_count).
    deferred_load_roms: Option<(Option<RomCacheKey>, Option<GetRoms>, u64)>,
    /// Brief “connected” banner after setup or when the server responds to heartbeat.
    startup_splash: Option<StartupSplash>,
}

impl App {
    /// Construct a new `App` with fresh cache and empty download list.
    pub fn new(
        client: RommClient,
        config: Config,
        registry: EndpointRegistry,
        server_version: Option<String>,
        startup_splash: Option<StartupSplash>,
    ) -> Self {
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
        }
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
                    if key.kind == KeyEventKind::Press && self.handle_key(key.code).await? {
                        break;
                    }
                }
            }

            // Process deferred ROM fetch (set during LibraryBrowse ↑/↓).
            // This avoids borrowing `self` mutably in two places at once:
            // the screen handler only *records* the intent to load ROMs,
            // and the actual HTTP call happens here after rendering.
            if let Some((key, req, expected)) = self.deferred_load_roms.take() {
                if let Ok(Some(roms)) = self.load_roms_cached(key, req, expected).await {
                    if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                        lib.set_roms(roms);
                    }
                }
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
    // Cache helper
    // -----------------------------------------------------------------------

    /// Fetch ROMs from the persistent cache if the count still matches,
    /// otherwise hit the API and update the cache on disk.
    async fn load_roms_cached(
        &mut self,
        key: Option<RomCacheKey>,
        req: Option<GetRoms>,
        expected_count: u64,
    ) -> Result<Option<RomList>> {
        // Try the disk-backed cache first.
        if let Some(k) = key {
            if let Some(cached) = self.rom_cache.get_valid(&k, expected_count) {
                return Ok(Some(cached.clone()));
            }
        }
        // Cache miss or stale — fetch fresh data from the API.
        if let Some(r) = req {
            let mut roms = self.client.call(&r).await?;
            let total = roms.total;
            let ceiling = 20000;

            // The RomM API has a default limit (often 500) even if we request more.
            // Loop until the items list is complete or we hit the ceiling.
            while (roms.items.len() as u64) < total && (roms.items.len() as u64) < ceiling {
                let mut next_req = r.clone();
                next_req.offset = Some(roms.items.len() as u32);

                let next_batch = self.client.call(&next_req).await?;
                if next_batch.items.is_empty() {
                    break;
                }
                roms.items.extend(next_batch.items);
            }

            if let Some(k) = key {
                self.rom_cache.insert(k, roms.clone(), expected_count); // also persists to disk
            }
            return Ok(Some(roms));
        }
        Ok(None)
    }

    // -----------------------------------------------------------------------
    // Key dispatch — one small method per screen
    // -----------------------------------------------------------------------

    async fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        if self.startup_splash.is_some() {
            self.startup_splash = None;
            return Ok(false);
        }

        // Global shortcut: 'd' toggles Download overlay (except on Search).
        if key == KeyCode::Char('d') && !matches!(&self.screen, AppScreen::Search(_)) {
            self.toggle_download_screen();
            return Ok(false);
        }

        match &self.screen {
            AppScreen::MainMenu(_) => self.handle_main_menu(key).await,
            AppScreen::LibraryBrowse(_) => self.handle_library_browse(key).await,
            AppScreen::Search(_) => self.handle_search(key).await,
            AppScreen::Settings(_) => self.handle_settings(key),
            AppScreen::Browse(_) => self.handle_browse(key),
            AppScreen::Execute(_) => self.handle_execute(key).await,
            AppScreen::Result(_) => self.handle_result(key),
            AppScreen::ResultDetail(_) => self.handle_result_detail(key),
            AppScreen::GameDetail(_) => self.handle_game_detail(key),
            AppScreen::Download(_) => self.handle_download(key),
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
                    let platforms = self.client.call(&ListPlatforms).await?;
                    let collections = self.client.call(&ListCollections).await.unwrap_or_default();
                    let mut lib = LibraryBrowseScreen::new(platforms, collections);
                    if lib.list_len() > 0 {
                        let key = lib.cache_key();
                        let expected = lib.expected_rom_count();
                        let req = lib
                            .get_roms_request_platform()
                            .or_else(|| lib.get_roms_request_collection());
                        if let Ok(Some(roms)) = self.load_roms_cached(key, req, expected).await {
                            lib.set_roms(roms);
                        }
                    }
                    self.screen = AppScreen::LibraryBrowse(lib);
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
                4 => self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone())),
                5 => return Ok(true),
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

        // If in search mode, intercept typing keys.
        if let Some(mode) = lib.search_mode {
            match key {
                KeyCode::Esc => lib.clear_search(),
                KeyCode::Backspace => lib.delete_search_char(),
                KeyCode::Char(c) => lib.add_search_char(c),
                KeyCode::Tab if mode == LibrarySearchMode::Jump => lib.jump_to_match(true),
                KeyCode::Enter => lib.search_mode = None, // Commit search (keep filtered/position)
                _ => {}
            }
            return Ok(false);
        }

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if lib.view_mode == LibraryViewMode::List {
                    lib.list_previous();
                    if lib.list_len() > 0 {
                        lib.clear_roms(); // avoid showing previous console's games
                        let key = lib.cache_key();
                        let expected = lib.expected_rom_count();
                        let req = lib
                            .get_roms_request_platform()
                            .or_else(|| lib.get_roms_request_collection());
                        self.deferred_load_roms = Some((key, req, expected));
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
                        let req = lib
                            .get_roms_request_platform()
                            .or_else(|| lib.get_roms_request_collection());
                        self.deferred_load_roms = Some((key, req, expected));
                    }
                } else {
                    lib.rom_next();
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if lib.view_mode == LibraryViewMode::Roms {
                    lib.back_to_list();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => lib.switch_view(),
            KeyCode::Tab => {
                if lib.view_mode == LibraryViewMode::List {
                    lib.switch_view();
                } else {
                    lib.switch_view(); // Normal tab also switches panels
                }
            }
            KeyCode::Char('/') if lib.view_mode == LibraryViewMode::Roms => {
                lib.enter_search(LibrarySearchMode::Filter);
            }
            KeyCode::Char('f') if lib.view_mode == LibraryViewMode::Roms => {
                lib.enter_search(LibrarySearchMode::Jump);
            }
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
            KeyCode::Char('t') => lib.switch_subsection(),
            KeyCode::Esc => {
                if lib.view_mode == LibraryViewMode::Roms {
                    lib.back_to_list();
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
                if search.result_groups.is_some() {
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
                } else if !search.query.is_empty() {
                    let req = GetRoms {
                        search_term: Some(search.query.clone()),
                        limit: Some(50),
                        ..Default::default()
                    };
                    if let Ok(roms) = self.client.call(&req).await {
                        search.set_results(roms);
                    }
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

    fn handle_settings(&mut self, key: KeyCode) -> Result<bool> {
        match key {
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
            KeyCode::Left | KeyCode::Char('h') => {
                if browse.view_mode == ViewMode::Endpoints {
                    browse.switch_view();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if browse.view_mode == ViewMode::Sections {
                    browse.switch_view();
                }
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
            KeyCode::Char('j') => {
                if result.view_mode == ResultViewMode::Json {
                    result.scroll_down(1);
                }
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
            KeyCode::Char('t') => {
                if result.table_row_count > 0 {
                    result.switch_view_mode();
                }
            }
            KeyCode::Enter => {
                if result.view_mode == ResultViewMode::Table && result.table_row_count > 0 {
                    if let Some(item) = result.get_selected_item_value() {
                        let prev = std::mem::replace(
                            &mut self.screen,
                            AppScreen::MainMenu(MainMenuScreen::new()),
                        );
                        if let AppScreen::Result(rs) = prev {
                            self.screen =
                                AppScreen::ResultDetail(ResultDetailScreen::new(rs, item));
                        }
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
            KeyCode::Enter => {
                // Only start a download once per detail view and avoid
                // stacking multiple concurrent downloads for the same ROM.
                if !detail.has_started_download {
                    detail.has_started_download = true;
                    self.downloads
                        .start_download(&detail.rom, self.client.clone());
                }
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
            AppScreen::Settings(settings) => settings.render(f, area),
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
        }
    }
}
