//! Main menu and startup update prompt handlers.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

use crate::core::startup_library_snapshot;

use super::super::{App, AppScreen};
use crate::tui::screens::{
    DownloadScreen, LibraryBrowseScreen, MainMenuScreen, SearchScreen, SettingsScreen,
};

impl App {
    pub(in crate::tui::app) async fn handle_startup_update_prompt(
        &mut self,
        key: &KeyEvent,
    ) -> Result<bool> {
        let Some(ref mut prompt) = self.startup_update_prompt else {
            return Ok(false);
        };
        if prompt.updating {
            return Ok(false); // Ignore keys while updating
        }

        match key.code {
            KeyCode::Char('u')
            | KeyCode::Char('U')
            | KeyCode::Char('y')
            | KeyCode::Char('Y')
            | KeyCode::Enter => {
                prompt.updating = true;
                Ok(false)
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if let Err(err) = crate::update::open_changelog_in_browser() {
                    self.global_error = Some(format!("Could not open changelog: {err:#}"));
                } else {
                    self.global_notice =
                        Some(format!("Opened changelog: {}", prompt.status.changelog_url));
                }
                Ok(false)
            }
            KeyCode::Esc
            | KeyCode::Char('s')
            | KeyCode::Char('S')
            | KeyCode::Char('n')
            | KeyCode::Char('N')
            | KeyCode::Char('q')
            | KeyCode::Char('Q') => {
                self.startup_update_prompt = None;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    // -- Main menu ----------------------------------------------------------

    pub(in crate::tui::app) async fn handle_main_menu(&mut self, key: &KeyEvent) -> Result<bool> {
        let menu = match &mut self.screen {
            AppScreen::MainMenu(m) => m,
            _ => return Ok(false),
        };
        match key.code {
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
                        self.queue_primary_rom_load(
                            key,
                            req,
                            expected,
                            "startup_first_selection",
                        );
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
            KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }
}
