//! Metadata match picker key handler and screen transitions.

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use romm_api::error::user_message_with_server_detail;

use super::super::{App, AppScreen};
use crate::tui::screens::metadata_match::{MetadataMatchPhase, MetadataMatchScreen};

impl App {
    pub(in crate::tui::app) fn metadata_edit_supported(&self) -> bool {
        self.metadata_edit_compat.supported
    }

    pub(in crate::tui::app) fn metadata_edit_blocked_message(&self) -> String {
        self.metadata_edit_compat.unsupported_message().to_string()
    }

    pub(in crate::tui::app) fn open_metadata_match_screen(&mut self) {
        if !self.metadata_edit_supported() {
            let msg = self.metadata_edit_blocked_message();
            if let AppScreen::GameDetail(detail) = &mut self.screen {
                detail.message = Some(msg);
                detail.message_clear_at = Some(Instant::now() + std::time::Duration::from_secs(5));
            }
            return;
        }
        let placeholder = self.transient_screen_placeholder();
        let prev = std::mem::replace(&mut self.screen, placeholder);
        let AppScreen::GameDetail(detail) = prev else {
            return;
        };
        self.screen = AppScreen::MetadataMatch(Box::new(MetadataMatchScreen::new_for_rom(detail)));
    }

    pub(in crate::tui::app) fn handle_metadata_match(&mut self, key: &KeyEvent) -> Result<bool> {
        let picker = match &mut self.screen {
            AppScreen::MetadataMatch(p) => p,
            _ => return Ok(false),
        };

        if matches!(picker.phase, MetadataMatchPhase::QueryInput) {
            return self.handle_metadata_match_query_input(key);
        }

        match key.code {
            KeyCode::Esc => {
                let placeholder = self.transient_screen_placeholder();
                let prev = std::mem::replace(&mut self.screen, placeholder);
                if let AppScreen::MetadataMatch(p) = prev {
                    self.screen = AppScreen::GameDetail(p.previous);
                }
            }
            KeyCode::Char('r')
                if matches!(
                    picker.phase,
                    MetadataMatchPhase::Ready | MetadataMatchPhase::Failed(_)
                ) =>
            {
                picker.return_to_query_input();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if matches!(picker.phase, MetadataMatchPhase::Ready) {
                    picker.move_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if matches!(picker.phase, MetadataMatchPhase::Ready) {
                    picker.move_down();
                }
            }
            KeyCode::Enter => {
                if !matches!(picker.phase, MetadataMatchPhase::Ready) {
                    return Ok(false);
                }
                let Some(fields) = picker.selected_update_fields() else {
                    return Ok(false);
                };
                if fields.match_fields.is_empty() {
                    let msg = "Selected result has no provider IDs to apply".to_string();
                    picker.phase = MetadataMatchPhase::Failed(msg.clone());
                    picker.message = Some(msg);
                    return Ok(false);
                }
                let rom_id = picker.previous.rom.id;
                let platform_id = picker.previous.rom.platform_id;
                picker.set_applying();
                self.spawn_metadata_apply_worker(rom_id, platform_id, fields, false);
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    fn handle_metadata_match_query_input(&mut self, key: &KeyEvent) -> Result<bool> {
        use crossterm::event::KeyModifiers;

        let picker = match &mut self.screen {
            AppScreen::MetadataMatch(p) => p,
            _ => return Ok(false),
        };

        match key.code {
            KeyCode::Esc => {
                let placeholder = self.transient_screen_placeholder();
                let prev = std::mem::replace(&mut self.screen, placeholder);
                if let AppScreen::MetadataMatch(p) = prev {
                    self.screen = AppScreen::GameDetail(p.previous);
                }
            }
            KeyCode::Enter => {
                let rom_id = picker.previous.rom.id;
                let term = picker.search_query.trim().to_string();
                if term.is_empty() {
                    picker.message = Some("Enter a search term".into());
                    return Ok(false);
                }
                picker.set_loading();
                self.spawn_metadata_search_worker(rom_id, term);
            }
            KeyCode::Backspace => picker.delete_char(),
            KeyCode::Delete => picker.delete_forward_char(),
            KeyCode::Left => picker.cursor_left(),
            KeyCode::Right => picker.cursor_right(),
            KeyCode::Home => picker.cursor_pos = 0,
            KeyCode::End => picker.cursor_pos = picker.search_query.len(),
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                picker.add_char(c)
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    pub(in crate::tui::app) fn spawn_metadata_unmatch(&mut self, rom_id: u64, platform_id: u64) {
        self.spawn_metadata_apply_worker(rom_id, platform_id, Default::default(), true);
    }

    fn finish_metadata_apply_success(&mut self, completed_rom_id: u64, rom: romm_api::types::Rom) {
        let applied = match &mut self.screen {
            AppScreen::MetadataMatch(picker)
                if picker.previous.rom.id == completed_rom_id && rom.id == completed_rom_id =>
            {
                picker.previous.apply_refreshed_rom(rom);
                let placeholder = self.transient_screen_placeholder();
                let prev = std::mem::replace(&mut self.screen, placeholder);
                if let AppScreen::MetadataMatch(p) = prev {
                    let mut detail = *p.previous;
                    detail.message = Some("Metadata updated.".into());
                    detail.message_clear_at = Some(Instant::now() + Duration::from_secs(3));
                    self.screen = AppScreen::GameDetail(Box::new(detail));
                }
                true
            }
            AppScreen::GameDetail(detail)
                if detail.rom.id == completed_rom_id && rom.id == completed_rom_id =>
            {
                detail.apply_refreshed_rom(rom);
                detail.metadata_unmatch_confirm = false;
                detail.message = Some("Metadata updated.".into());
                detail.message_clear_at = Some(Instant::now() + Duration::from_secs(3));
                true
            }
            _ => false,
        };
        if applied {
            self.force_rom_reload_after_metadata = true;
            self.maybe_start_game_detail_cover_load();
            self.refresh_current_game_achievements();
        }
    }

    pub(in crate::tui::app) fn apply_metadata_search_complete(
        &mut self,
        done: super::super::background::types::MetadataSearchDone,
    ) {
        if let AppScreen::MetadataMatch(picker) = &mut self.screen {
            if picker.previous.rom.id != done.rom_id {
                return;
            }
            match done.result {
                Ok(rows) => picker.apply_search_result(rows),
                Err(e) => picker.apply_search_error(user_message_with_server_detail(&e, 200)),
            }
        }
    }

    pub(in crate::tui::app) fn apply_metadata_apply_complete(
        &mut self,
        done: super::super::background::types::MetadataApplyDone,
    ) {
        match done.result {
            Ok(rom) => self.finish_metadata_apply_success(done.rom_id, *rom),
            Err(e) => {
                let msg = format!(
                    "Metadata update failed: {}",
                    user_message_with_server_detail(&e, 200)
                );
                match &mut self.screen {
                    AppScreen::MetadataMatch(picker) if picker.previous.rom.id == done.rom_id => {
                        picker.phase = MetadataMatchPhase::Failed(msg.clone());
                        picker.message = Some(msg);
                    }
                    AppScreen::GameDetail(detail) if detail.rom.id == done.rom_id => {
                        detail.metadata_unmatch_confirm = false;
                        detail.message = Some(msg);
                        detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                    }
                    _ => {}
                }
            }
        }
    }
}
