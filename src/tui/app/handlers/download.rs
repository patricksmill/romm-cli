//! Download overlay key handlers.

use crossterm::event::{KeyCode, KeyEvent};

use super::super::{App, AppScreen};
use crate::tui::screens::{DownloadScreen, MainMenuScreen};

impl App {
    pub(in crate::tui::app) fn toggle_download_screen(&mut self) {
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
                self.screen = AppScreen::Download(DownloadScreen::new(
                    self.downloads.shared(),
                    self.downloads.shared_extras(),
                ));
            }
        }
    }
}

pub(in crate::tui::app) fn map_download_key(key: &KeyEvent) -> Vec<super::super::event::Action> {
    use super::super::event::Action;
    if key.code == KeyCode::Esc || key.code == KeyCode::Char('d') {
        vec![Action::CloseDownloadOverlay]
    } else {
        Vec::new()
    }
}
