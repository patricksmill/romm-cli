//! Main menu and startup update prompt handlers.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

use crate::core::startup_library_snapshot;

use super::super::{App, AppScreen};
use crate::tui::screens::{
    DownloadScreen, LibraryBrowseScreen, MainMenuScreen, SearchScreen, SettingsScreen,
};

pub(in crate::tui::app) fn map_main_menu_key(key: &KeyEvent) -> Vec<super::super::event::Action> {
    use super::super::event::Action;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => vec![Action::MainMenuPrevious],
        KeyCode::Down | KeyCode::Char('j') => vec![Action::MainMenuNext],
        KeyCode::Enter => vec![Action::MainMenuActivate],
        KeyCode::Esc | KeyCode::Char('q') => vec![Action::Quit],
        _ => Vec::new(),
    }
}

impl App {
    pub(in crate::tui::app) async fn apply_main_menu_action(
        &mut self,
        action: super::super::event::Action,
    ) -> Result<bool> {
        use super::super::event::Action;
        let menu = match &mut self.screen {
            AppScreen::MainMenu(m) => m,
            _ => return Ok(false),
        };
        match action {
            Action::MainMenuPrevious => menu.previous(),
            Action::MainMenuNext => menu.next(),
            Action::MainMenuActivate => match menu.selected {
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
                        self.queue_primary_rom_load(key, req, expected, "startup_first_selection");
                    }
                    self.screen = AppScreen::LibraryBrowse(Box::new(lib));
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
                    self.screen = AppScreen::Download(DownloadScreen::new(
                        self.downloads.shared(),
                        self.downloads.shared_extras(),
                    ));
                }
                3 => {
                    self.screen = AppScreen::Settings(Box::new(SettingsScreen::new(
                        &self.config,
                        self.server_version.as_deref(),
                        self.save_sync_compat.clone(),
                    )))
                }
                4 => return Ok(true),
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }
}
