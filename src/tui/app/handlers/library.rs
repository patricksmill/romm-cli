//! Library browse key handler.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::Path;
use std::time::Instant;

use super::super::{App, AppScreen};
use crate::tui::screens::{GameDetailPrevious, GameDetailScreen, MainMenuScreen};

impl App {
    pub(in crate::tui::app) async fn handle_library_browse(
        &mut self,
        key: &KeyEvent,
    ) -> Result<bool> {
        use crate::tui::path_picker::PathPickerEvent;
        use crate::tui::screens::library_browse::{LibrarySearchMode, LibraryViewMode};

        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            if lib.upload_prompt.is_some() {
                if let Some(up) = lib.upload_prompt.as_mut() {
                    if key.code == KeyCode::Esc {
                        lib.close_upload_prompt();
                        return Ok(false);
                    }
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
                    {
                        up.scan_after = !up.scan_after;
                        return Ok(false);
                    }
                    match up.picker.handle_key(key) {
                        PathPickerEvent::Confirmed(path) => {
                            let scan_after = up.scan_after;
                            if !Path::new(&path).is_file() {
                                lib.set_metadata_footer(Some(format!(
                                    "Not a file: {}",
                                    path.display()
                                )));
                                return Ok(false);
                            }
                            let Some(pid) = lib.selected_platform_id() else {
                                lib.set_metadata_footer(Some(
                                    "Select a console before uploading.".into(),
                                ));
                                return Ok(false);
                            };
                            lib.close_upload_prompt();
                            self.spawn_library_upload_worker(pid, path, scan_after);
                        }
                        PathPickerEvent::None => {}
                    }
                }
                return Ok(false);
            }
        }

        if self.library_upload_inflight {
            return Ok(false);
        }

        let lib = match &mut self.screen {
            AppScreen::LibraryBrowse(l) => l,
            _ => return Ok(false),
        };

        // List pane: search typing bar
        if lib.view_mode == LibraryViewMode::List {
            if let Some(mode) = lib.list_search.mode {
                let old_key = lib.cache_key();
                match key.code {
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
                match key.code {
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

        match key.code {
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
                            == crate::tui::screens::library_browse::LibrarySubsection::ByCollection
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
                            == crate::tui::screens::library_browse::LibrarySubsection::ByCollection
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
                        self.maybe_start_game_detail_cover_load();
                        self.refresh_current_game_saves();
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
                if lib.subsection
                    == crate::tui::screens::library_browse::LibrarySubsection::ByCollection
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
}
