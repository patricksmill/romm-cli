//! Library startup and overlay restore helpers.

use std::time::Instant;

use crate::core::startup_library_snapshot;

use super::super::{App, AppScreen};
use crate::tui::screens::LibraryBrowseScreen;

impl App {
    pub(in crate::tui::app) fn is_overlay_screen(screen: &AppScreen) -> bool {
        matches!(
            screen,
            AppScreen::Search(_) | AppScreen::Settings(_) | AppScreen::Download(_)
        )
    }

    /// Short-lived placeholder during `mem::replace` screen swaps.
    pub(in crate::tui::app) fn transient_screen_placeholder(&self) -> AppScreen {
        AppScreen::LibraryBrowse(Box::new(LibraryBrowseScreen::new(
            Vec::new(),
            Vec::new(),
            self.config.tui_layout.library_left_panel_percent,
        )))
    }

    /// Open library browse (startup home) with snapshot preload and metadata refresh.
    pub fn open_library_browse(&mut self) {
        let start = Instant::now();
        let snap = startup_library_snapshot::load_snapshot();
        let (platforms, collections, from_disk) = match snap {
            Some(s) => (s.platforms, s.collections, true),
            None => (Vec::new(), Vec::new(), false),
        };
        let mut lib = LibraryBrowseScreen::new(
            platforms,
            collections,
            self.config.tui_layout.library_left_panel_percent,
        );
        if from_disk && lib.list_len() > 0 {
            lib.set_metadata_footer(Some("Refreshing library metadata in background…".into()));
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

    pub(in crate::tui::app) fn restore_screen_or_library(&mut self, stored: Option<AppScreen>) {
        self.screen = stored.unwrap_or_else(|| {
            let mut lib = LibraryBrowseScreen::new(
                Vec::new(),
                Vec::new(),
                self.config.tui_layout.library_left_panel_percent,
            );
            lib.set_metadata_footer(Some("Loading library metadata…".into()));
            AppScreen::LibraryBrowse(Box::new(lib))
        });
        if matches!(self.screen, AppScreen::LibraryBrowse(_)) {
            self.spawn_library_metadata_refresh();
        }
    }
}
