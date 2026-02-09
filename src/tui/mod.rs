pub mod cache;
pub mod download;
pub mod openapi;
pub mod screens;
pub mod utils;

use anyhow::{anyhow, Result};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const DEBUG_LOG: &str = "c:\\Users\\Patri\\Projects\\romm-cli\\.cursor\\debug.log";

/// Max ROMs to fetch per console/collection so the games list is scrollable (not just the visible rows).
const LIBRARY_FETCH_LIMIT: u32 = 500;

use crate::client::RommClient;
use crate::config::Config;
use crate::endpoints::{collections::ListCollections, platforms::ListPlatforms, roms::GetRoms};
use crate::types::RomList;

use self::cache::RomCacheKey;
use self::download::{DownloadJob, DownloadStatus};

use self::screens::{
    BrowseScreen, DownloadScreen, ExecuteScreen, GameDetailPrevious, GameDetailScreen,
    LibraryBrowseScreen, MainMenuScreen, ResultDetailScreen, ResultScreen, SearchScreen,
    SettingsScreen,
};

pub enum AppScreen {
    MainMenu(MainMenuScreen),
    LibraryBrowse(LibraryBrowseScreen),
    Search(SearchScreen),
    Settings(SettingsScreen),
    Browse(BrowseScreen),
    Execute(ExecuteScreen),
    Result(ResultScreen),
    ResultDetail(ResultDetailScreen),
    GameDetail(GameDetailScreen),
    Download(DownloadScreen),
}

pub struct App {
    pub screen: AppScreen,
    pub client: RommClient,
    pub config: Config,
    pub registry: openapi::EndpointRegistry,
    pub should_quit: bool,
    /// Games panel height (rows) for lazy-load limit; set when rendering LibraryBrowse.
    pub library_visible_rows: usize,
    /// ROMs per console/collection so revisiting is instant.
    pub rom_cache: HashMap<RomCacheKey, RomList>,
    /// Deferred ROM load (key, req) so we can run it after releasing the screen borrow.
    pub deferred_load_roms: Option<(Option<RomCacheKey>, Option<GetRoms>)>,
    /// Active and recent downloads for the Download screen.
    pub active_downloads: Arc<Mutex<Vec<DownloadJob>>>,
    /// Screen to restore when closing the Download screen (d or Esc).
    pub screen_before_download: Option<AppScreen>,
}

impl App {
    pub fn new(client: RommClient, config: Config, registry: openapi::EndpointRegistry) -> Self {
        Self {
            screen: AppScreen::MainMenu(MainMenuScreen::new()),
            client,
            config,
            registry,
            should_quit: false,
            library_visible_rows: 20,
            rom_cache: HashMap::new(),
            deferred_load_roms: None,
            active_downloads: Arc::new(Mutex::new(Vec::new())),
            screen_before_download: None,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.handle_key(key.code).await? {
                        break;
                    }
                }
            }

            // #region agent log
            if self.deferred_load_roms.is_some() {
                let _ = OpenOptions::new().append(true).open(DEBUG_LOG).and_then(|mut f| {
                    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    writeln!(f, r#"{{"sessionId":"debug-session","hypothesisId":"H1,H5","location":"tui/mod.rs:run_loop","message":"deferred pending","data":{{"has_deferred":true}},"timestamp":{}}}"#, ts)
                });
            }
            // #endregion

            if let Some((key, req)) = self.deferred_load_roms.take() {
                if let Ok(Some(roms)) = self.load_roms_for_library(key, req).await {
                    let n = roms.items.len();
                    if let AppScreen::LibraryBrowse(ref mut lib) = &mut self.screen {
                        lib.set_roms(roms);
                        let _ = OpenOptions::new().append(true).open(DEBUG_LOG).and_then(|mut f| {
                            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                            writeln!(f, r#"{{"sessionId":"debug-session","runId":"post-fix","hypothesisId":"H3","location":"tui/mod.rs:run_loop","message":"processed deferred set_roms","data":{{"roms_count":{}}},"timestamp":{}}}"#, n, ts)
                        });
                    }
                }
            }

            if self.should_quit {
                break;
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

    /// Load ROMs for library browse: use cache if present, else fetch and insert.
    async fn load_roms_for_library(
        &mut self,
        key: Option<RomCacheKey>,
        req: Option<GetRoms>,
    ) -> Result<Option<RomList>> {
        if let Some(k) = key {
            if let Some(cached) = self.rom_cache.get(&k) {
                return Ok(Some(cached.clone()));
            }
        }
        if let Some(r) = req {
            let roms = self.client.call(&r).await?;
            if let Some(k) = key {
                self.rom_cache.insert(k, roms.clone());
            }
            return Ok(Some(roms));
        }
        Ok(None)
    }

    async fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        if key == KeyCode::Char('d') && !matches!(&self.screen, AppScreen::Search(_)) {
            let current =
                std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
            match current {
                AppScreen::Download(_) => {
                    self.screen = self.screen_before_download.take().unwrap_or_else(|| {
                        AppScreen::MainMenu(MainMenuScreen::new())
                    });
                }
                other => {
                    self.screen_before_download = Some(other);
                    self.screen =
                        AppScreen::Download(DownloadScreen::new(self.active_downloads.clone()));
                }
            }
            return Ok(false);
        }

        match &mut self.screen {
            AppScreen::MainMenu(menu) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => menu.previous(),
                    KeyCode::Down | KeyCode::Char('j') => menu.next(),
                    KeyCode::Enter => {
                        match menu.selected {
                            0 => {
                                let platforms = self.client.call(&ListPlatforms::default()).await?;
                                let collections = self
                                    .client
                                    .call(&ListCollections::default())
                                    .await
                                    .unwrap_or_default();
                                let mut lib = LibraryBrowseScreen::new(platforms, collections);
                                if lib.list_len() > 0 {
                                    let limit = LIBRARY_FETCH_LIMIT;
                                    let key = lib.cache_key();
                                    let req = lib
                                        .get_roms_request_platform_with_limit(limit)
                                        .or_else(|| lib.get_roms_request_collection_with_limit(limit));
                                    if let Ok(Some(roms)) = self.load_roms_for_library(key, req).await {
                                        lib.set_roms(roms);
                                    }
                                }
                                self.screen = AppScreen::LibraryBrowse(lib);
                            }
                            1 => {
                                self.screen = AppScreen::Search(SearchScreen::new());
                            }
                            2 => {
                                self.screen =
                                    AppScreen::Settings(SettingsScreen::new(&self.config));
                            }
                            3 => {
                                self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
                            }
                            4 => return Ok(true),
                            _ => {}
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
            AppScreen::LibraryBrowse(lib) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if lib.view_mode == screens::library_browse::LibraryViewMode::List {
                            lib.list_previous();
                            if lib.list_len() > 0 {
                                let limit = LIBRARY_FETCH_LIMIT;
                                let key = lib.cache_key();
                                let req = lib
                                    .get_roms_request_platform_with_limit(limit)
                                    .or_else(|| lib.get_roms_request_collection_with_limit(limit));
                                // #region agent log
                                let _ = OpenOptions::new().append(true).open(DEBUG_LOG).and_then(|mut f| {
                                    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                                    let key_s = format!("{:?}", key).replace('"', "\\\"");
                                    writeln!(f, r#"{{"sessionId":"debug-session","hypothesisId":"H1,H4","location":"tui/mod.rs:Up","message":"deferred set","data":{{"list_index":{},"key":"{}","req_platform_id":{:?}}},"timestamp":{}}}"#, lib.list_index, key_s, req.as_ref().and_then(|r| r.platform_id), ts)
                                });
                                // #endregion
                                self.deferred_load_roms = Some((key, req));
                            }
                        } else {
                            lib.rom_previous();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if lib.view_mode == screens::library_browse::LibraryViewMode::List {
                            lib.list_next();
                            if lib.list_len() > 0 {
                                let limit = LIBRARY_FETCH_LIMIT;
                                let key = lib.cache_key();
                                let req = lib
                                    .get_roms_request_platform_with_limit(limit)
                                    .or_else(|| lib.get_roms_request_collection_with_limit(limit));
                                // #region agent log
                                let _ = OpenOptions::new().append(true).open(DEBUG_LOG).and_then(|mut f| {
                                    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                                    let key_s = format!("{:?}", key).replace('"', "\\\"");
                                    writeln!(f, r#"{{"sessionId":"debug-session","hypothesisId":"H1,H4","location":"tui/mod.rs:Down","message":"deferred set","data":{{"list_index":{},"key":"{}","req_platform_id":{:?}}},"timestamp":{}}}"#, lib.list_index, key_s, req.as_ref().and_then(|r| r.platform_id), ts)
                                });
                                // #endregion
                                self.deferred_load_roms = Some((key, req));
                            }
                        } else {
                            lib.rom_next();
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if lib.view_mode == screens::library_browse::LibraryViewMode::Roms {
                            lib.back_to_list();
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                        if lib.view_mode == screens::library_browse::LibraryViewMode::List {
                            lib.switch_view();
                        } else if lib.view_mode == screens::library_browse::LibraryViewMode::Roms {
                            lib.switch_view();
                        }
                    }
                    KeyCode::Enter => {
                        if lib.view_mode == screens::library_browse::LibraryViewMode::List {
                            lib.switch_view();
                        } else if let Some((primary, others)) = lib.get_selected_group() {
                            let lib_screen = std::mem::replace(
                                &mut self.screen,
                                AppScreen::MainMenu(MainMenuScreen::new()),
                            );
                            if let AppScreen::LibraryBrowse(l) = lib_screen {
                                self.screen = AppScreen::GameDetail(GameDetailScreen::new(
                                    primary,
                                    others,
                                    GameDetailPrevious::Library(l),
                                ));
                            }
                        }
                    }
                    KeyCode::Char('t') => lib.switch_subsection(),
                    KeyCode::Esc => {
                        if lib.view_mode == screens::library_browse::LibraryViewMode::Roms {
                            lib.back_to_list();
                        } else {
                            self.screen = AppScreen::MainMenu(MainMenuScreen::new());
                        }
                    }
                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
            AppScreen::Search(search) => {
                match key {
                    KeyCode::Backspace => search.delete_char(),
                    KeyCode::Left => search.cursor_left(),
                    KeyCode::Right => search.cursor_right(),
                    KeyCode::Up | KeyCode::Char('k') => search.previous(),
                    KeyCode::Down | KeyCode::Char('j') => search.next(),
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char(c) => search.add_char(c),
                    KeyCode::Enter => {
                        if search.result_groups.is_some() {
                            if let Some((primary, others)) = search.get_selected_group() {
                                let search_screen = std::mem::replace(
                                    &mut self.screen,
                                    AppScreen::MainMenu(MainMenuScreen::new()),
                                );
                                if let AppScreen::Search(s) = search_screen {
                                    self.screen = AppScreen::GameDetail(GameDetailScreen::new(
                                        primary,
                                        others,
                                        GameDetailPrevious::Search(s),
                                    ));
                                }
                            }
                        } else if !search.query.is_empty() {
                            let query = search.query.clone();
                            let req = GetRoms {
                                search_term: Some(query),
                                limit: Some(50),
                                ..Default::default()
                            };
                            match self.client.call(&req).await {
                                Ok(roms) => search.set_results(roms),
                                Err(_) => {}
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
                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
            AppScreen::Settings(_) => {
                match key {
                    KeyCode::Esc => {
                        self.screen = AppScreen::MainMenu(MainMenuScreen::new());
                    }
                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
            AppScreen::Browse(browse) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => browse.previous(),
                    KeyCode::Down | KeyCode::Char('j') => browse.next(),
                    KeyCode::Left | KeyCode::Char('h') => {
                        if browse.view_mode == screens::browse::ViewMode::Endpoints {
                            browse.switch_view();
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if browse.view_mode == screens::browse::ViewMode::Sections {
                            browse.switch_view();
                        }
                    }
                    KeyCode::Tab => browse.switch_view(),
                    KeyCode::Enter => {
                        if browse.view_mode == screens::browse::ViewMode::Endpoints {
                            if let Some(endpoint) = browse.get_selected_endpoint() {
                                let endpoint_clone = endpoint.clone();
                                self.screen = AppScreen::Execute(ExecuteScreen::new(endpoint_clone));
                            }
                        } else {
                            browse.switch_view();
                        }
                    }
                    KeyCode::Esc => {
                        self.screen = AppScreen::MainMenu(MainMenuScreen::new());
                    }
                    _ => {}
                }
            }
            AppScreen::Execute(execute) => {
                match key {
                    KeyCode::Tab => execute.next_field(),
                    KeyCode::BackTab => execute.previous_field(),
                    KeyCode::Char(c) => execute.add_char_to_focused(c),
                    KeyCode::Backspace => execute.delete_char_from_focused(),
                    KeyCode::Enter => {
                        let endpoint = execute.endpoint.clone();
                        let query = execute.get_query_params();
                        let body = if execute.endpoint.has_body && !execute.body_text.is_empty() {
                            Some(serde_json::from_str(&execute.body_text)?)
                        } else {
                            None
                        };

                        match self
                            .client
                            .request_json(&endpoint.method, &endpoint.path, &query, body)
                            .await
                        {
                            Ok(result) => {
                                self.screen = AppScreen::Result(ResultScreen::new(
                                    result,
                                    Some(&endpoint.method),
                                    Some(&endpoint.path),
                                ));
                            }
                            Err(e) => {
                                let error_json = serde_json::json!({
                                    "error": format!("{}", e)
                                });
                                self.screen = AppScreen::Result(ResultScreen::new(
                                    error_json,
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
            }
            AppScreen::Result(result) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if result.view_mode == screens::result::ResultViewMode::Json {
                            result.scroll_up(1);
                        } else {
                            result.table_previous();
                        }
                    }
                    KeyCode::Down => {
                        if result.view_mode == screens::result::ResultViewMode::Json {
                            result.scroll_down(1);
                        } else {
                            result.table_next();
                        }
                    }
                    KeyCode::Char('j') => {
                        if result.view_mode == screens::result::ResultViewMode::Json {
                            result.scroll_down(1);
                        }
                    }
                    KeyCode::PageUp => {
                        if result.view_mode == screens::result::ResultViewMode::Table {
                            result.table_page_up();
                        } else {
                            result.scroll_up(10);
                        }
                    }
                    KeyCode::PageDown => {
                        if result.view_mode == screens::result::ResultViewMode::Table {
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
                        if result.view_mode == screens::result::ResultViewMode::Table
                            && result.table_row_count > 0
                        {
                            if let Some(item) = result.get_selected_item_value() {
                                let result_screen = std::mem::replace(
                                    &mut self.screen,
                                    AppScreen::MainMenu(MainMenuScreen::new()),
                                );
                                if let AppScreen::Result(rs) = result_screen {
                                    self.screen = AppScreen::ResultDetail(
                                        ResultDetailScreen::new(rs, item),
                                    );
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
            }
            AppScreen::ResultDetail(detail) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => detail.scroll_up(1),
                    KeyCode::Down | KeyCode::Char('j') => detail.scroll_down(1),
                    KeyCode::PageUp => detail.scroll_up(10),
                    KeyCode::PageDown => detail.scroll_down(10),
                    KeyCode::Char('o') => detail.open_image_url(),
                    KeyCode::Esc => {
                        detail.clear_message();
                        let detail_screen = std::mem::replace(
                            &mut self.screen,
                            AppScreen::MainMenu(MainMenuScreen::new()),
                        );
                        if let AppScreen::ResultDetail(d) = detail_screen {
                            self.screen = AppScreen::Result(d.parent);
                        }
                    }
                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
            AppScreen::GameDetail(detail) => {
                match key {
                    KeyCode::Enter => {
                        let platform = detail
                            .rom
                            .platform_display_name
                            .as_deref()
                            .or(detail.rom.platform_custom_name.as_deref())
                            .unwrap_or("—")
                            .to_string();
                        let job = DownloadJob::new(
                            detail.rom.id,
                            detail.rom.name.clone(),
                            platform,
                        );
                        let job_id = job.id;
                        let rom_id = detail.rom.id;
                        let fs_name = detail.rom.fs_name.clone();
                        self.active_downloads.lock().unwrap().push(job);
                        let client = self.client.clone();
                        let downloads = self.active_downloads.clone();
                        tokio::spawn(async move {
                            let save_dir = Path::new("./downloads");
                            let _ = std::fs::create_dir_all(save_dir);
                            let base = sanitize_download_filename(&fs_name);
                            let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(&base);
                            let filename = format!("{}.zip", stem);
                            let save_path = save_dir.join(filename);
                            let on_progress = |received: u64, total: u64| {
                                let p = if total > 0 {
                                    received as f64 / total as f64
                                } else {
                                    0.0
                                };
                                if let Ok(mut list) = downloads.lock() {
                                    if let Some(job) = list.iter_mut().find(|j| j.id == job_id) {
                                        job.progress = p;
                                    }
                                }
                            };
                            match client.download_rom(rom_id, &save_path, on_progress).await {
                                Ok(()) => {
                                    if let Ok(mut list) = downloads.lock() {
                                        if let Some(job) = list.iter_mut().find(|j| j.id == job_id)
                                        {
                                            job.status = DownloadStatus::Done;
                                            job.progress = 1.0;
                                        }
                                    }
                                }
                                Err(e) => {
                                    if let Ok(mut list) = downloads.lock() {
                                        if let Some(job) =
                                            list.iter_mut().find(|j| j.id == job_id)
                                        {
                                            job.status =
                                                DownloadStatus::Error(e.to_string());
                                        }
                                    }
                                }
                            }
                        });
                        detail.message = Some("Download started".to_string());
                    }
                    KeyCode::Char('o') => detail.open_cover(),
                    KeyCode::Char('m') => detail.toggle_technical(),
                    KeyCode::Esc => {
                        detail.clear_message();
                        let detail_screen = std::mem::replace(
                            &mut self.screen,
                            AppScreen::MainMenu(MainMenuScreen::new()),
                        );
                        if let AppScreen::GameDetail(g) = detail_screen {
                            self.screen = match g.previous {
                                GameDetailPrevious::Library(l) => AppScreen::LibraryBrowse(l),
                                GameDetailPrevious::Search(s) => AppScreen::Search(s),
                            };
                        }
                    }
                    _ => {
                        if key == KeyCode::Char('q') {
                            return Ok(true);
                        }
                    }
                }
            }
            AppScreen::Download(_) => {
                if key == KeyCode::Esc || key == KeyCode::Char('d') {
                    let _ = std::mem::replace(
                        &mut self.screen,
                        AppScreen::MainMenu(MainMenuScreen::new()),
                    );
                    self.screen = self.screen_before_download.take().unwrap_or_else(|| {
                        AppScreen::MainMenu(MainMenuScreen::new())
                    });
                }
            }
        }

        Ok(false)
    }

    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = f.size();
        match &mut self.screen {
            AppScreen::MainMenu(menu) => menu.render(f, area),
            AppScreen::LibraryBrowse(lib) => {
                let chunks = ratatui::layout::Layout::default()
                    .constraints([
                        ratatui::layout::Constraint::Percentage(30),
                        ratatui::layout::Constraint::Percentage(70),
                    ])
                    .direction(ratatui::layout::Direction::Horizontal)
                    .split(area);
                let right_chunks = ratatui::layout::Layout::default()
                    .constraints([
                        ratatui::layout::Constraint::Min(5),
                        ratatui::layout::Constraint::Length(3),
                    ])
                    .direction(ratatui::layout::Direction::Vertical)
                    .split(chunks[1]);
                self.library_visible_rows = (right_chunks[0].height as usize).saturating_sub(3).max(1);
                lib.render(f, area);
            }
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

fn sanitize_download_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ' {
                c
            } else if c == '/' || c == '\\' {
                '_'
            } else {
                '_'
            }
        })
        .collect()
}

pub async fn run(client: RommClient, config: Config) -> Result<()> {
    let openapi_path = std::env::var("ROMM_OPENAPI_PATH")
        .unwrap_or_else(|_| "openapi.json".to_string());

    let registry = if std::path::Path::new(&openapi_path).exists() {
        openapi::EndpointRegistry::from_file(&openapi_path)?
    } else {
        return Err(anyhow!(
            "OpenAPI file not found at {}. Please provide openapi.json or set ROMM_OPENAPI_PATH environment variable.",
            openapi_path
        ));
    };

    let mut app = App::new(client, config, registry);
    app.run().await?;

    Ok(())
}
