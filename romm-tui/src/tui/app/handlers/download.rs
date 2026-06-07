//! Download overlay key handlers.

use crossterm::event::{KeyCode, KeyEvent};

use super::super::{App, AppScreen};
use crate::tui::screens::DownloadScreen;

impl App {
    pub(in crate::tui::app) fn toggle_download_screen(&mut self) {
        match &self.screen {
            AppScreen::Download(_) => {
                let stored = self.screen_before_download.take();
                self.restore_screen_or_library(stored);
            }
            _ => {
                let prev = std::mem::replace(
                    &mut self.screen,
                    AppScreen::Download(DownloadScreen::new(
                        self.downloads.shared(),
                        self.downloads.shared_extras(),
                    )),
                );
                if !Self::is_overlay_screen(&prev) {
                    self.screen_before_download = Some(prev);
                }
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
