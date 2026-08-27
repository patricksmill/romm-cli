//! Game detail and extras picker key handlers.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use romm_api::config::resolve_game_save_dir;
use romm_api::core::extras::has_update_or_dlc_extras;
use romm_api::error::{ApiError, RommError};

use super::super::background::types::{SaveDownloadDone, SaveUploadDone};
use super::super::{App, AppScreen};
use crate::tui::screens::game_detail::DetailTab;
use crate::tui::screens::GameDetailPrevious;

fn safe_path_segment(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        "game".to_string()
    } else {
        trimmed.to_string()
    }
}

fn reserve_unique_save_path(dir: &Path, file_name: &str) -> PathBuf {
    let safe_name = safe_path_segment(file_name);
    let base = Path::new(&safe_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("save");
    let ext = Path::new(&safe_name).extension().and_then(|s| s.to_str());
    let mut candidate = dir.join(&safe_name);
    let mut n = 1u32;
    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(_) => return candidate,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return candidate,
        }

        let name = match ext {
            Some(ext) if !ext.is_empty() => format!("{base}-{n}.{ext}"),
            _ => format!("{base}-{n}"),
        };
        candidate = dir.join(name);
        n += 1;
    }
}

impl App {
    fn prompt_metadata_unmatch(&mut self) -> Result<bool> {
        use std::time::{Duration, Instant};
        let supported = self.metadata_edit_supported();
        let blocked = self.metadata_edit_blocked_message();
        if let AppScreen::GameDetail(detail) = &mut self.screen {
            if !supported {
                detail.message = Some(blocked);
                detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
            } else {
                detail.metadata_unmatch_confirm = true;
                detail.message = Some("Unmatch metadata? Press y to confirm, n to cancel.".into());
                detail.message_clear_at = None;
            }
        }
        Ok(false)
    }

    fn handle_metadata_unmatch_confirm_key(&mut self, key: &KeyEvent) -> Result<bool> {
        use std::time::{Duration, Instant};
        let supported = self.metadata_edit_supported();
        let blocked = self.metadata_edit_blocked_message();
        let AppScreen::GameDetail(detail) = &mut self.screen else {
            return Ok(false);
        };
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if !supported {
                    detail.message = Some(blocked);
                    detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                    detail.metadata_unmatch_confirm = false;
                    return Ok(false);
                }
                let rom_id = detail.rom.id;
                let platform_id = detail.rom.platform_id;
                detail.metadata_unmatch_confirm = false;
                detail.message = Some("Removing metadata match…".into());
                detail.message_clear_at = None;
                self.spawn_metadata_unmatch(rom_id, platform_id);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                detail.metadata_unmatch_confirm = false;
                detail.clear_message();
            }
            _ => {}
        }
        Ok(false)
    }

    pub(in crate::tui::app) fn handle_game_detail(&mut self, key: &KeyEvent) -> Result<bool> {
        use crate::tui::path_picker::PathPickerEvent;
        if !matches!(self.screen, AppScreen::GameDetail(_)) {
            return Ok(false);
        }

        if key.code == KeyCode::Char('m') {
            let is_identified = match &self.screen {
                AppScreen::GameDetail(d) => d.rom.is_identified,
                _ => false,
            };
            if is_identified {
                return self.prompt_metadata_unmatch();
            } else {
                self.open_metadata_match_screen();
                return Ok(false);
            }
        }

        if let AppScreen::GameDetail(detail) = &mut self.screen {
            if detail.metadata_unmatch_confirm {
                return self.handle_metadata_unmatch_confirm_key(key);
            }
        }

        let detail = match &mut self.screen {
            AppScreen::GameDetail(d) => d,
            _ => return Ok(false),
        };

        if let Some(picker) = detail.save_upload_picker.as_mut() {
            if key.code == KeyCode::Esc {
                detail.save_upload_picker = None;
                detail.clear_message();
                return Ok(false);
            }
            match picker.handle_key(key) {
                PathPickerEvent::Confirmed(path) => {
                    let rom_id = detail.rom.id;
                    detail.save_upload_picker = None;
                    detail.message = Some("Uploading save...".into());
                    detail.message_clear_at = None;
                    let client = self.client.clone();
                    let tx = self.save_upload_tx.clone();
                    tokio::spawn(async move {
                        let result = client
                            .upload_save_file(rom_id, None, &path)
                            .await
                            .map(|_| ())
                            .map_err(RommError::from);
                        let _ = tx.send(SaveUploadDone { rom_id, result });
                    });
                }
                PathPickerEvent::None => {}
            }
            return Ok(false);
        }

        // Acknowledge download completion on any key press
        // (check if there's a completed/errored download for this ROM)
        if !detail.download_completion_acknowledged {
            if let Ok(list) = detail.downloads.lock() {
                let has_completed = list.iter().any(|j| {
                    j.rom_id == detail.rom.id
                        && matches!(
                            j.status,
                            romm_api::core::download::DownloadStatus::Done
                                | romm_api::core::download::DownloadStatus::SkippedAlreadyExists
                                | romm_api::core::download::DownloadStatus::Cancelled
                                | romm_api::core::download::DownloadStatus::FinalizeFailed(_)
                                | romm_api::core::download::DownloadStatus::Error(_)
                        )
                });
                let is_still_downloading = list.iter().any(|j| {
                    j.rom_id == detail.rom.id
                        && matches!(
                            j.status,
                            romm_api::core::download::DownloadStatus::Downloading
                        )
                });
                // Only acknowledge if there's a completion and no active download
                if has_completed && !is_still_downloading {
                    detail.download_completion_acknowledged = true;
                }
            }
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Left => {
                    detail.adjust_cover_panel_width(-2);
                    self.config.tui_layout.game_detail_cover_panel_width = detail.cover_panel_width;
                    self.persist_tui_layout();
                    return Ok(false);
                }
                KeyCode::Right => {
                    detail.adjust_cover_panel_width(2);
                    self.config.tui_layout.game_detail_cover_panel_width = detail.cover_panel_width;
                    self.persist_tui_layout();
                    return Ok(false);
                }
                _ => {}
            }
        }

        let mut trigger_screenshot = false;
        match key.code {
            KeyCode::Char('1') => detail.select_tab(DetailTab::Info),
            KeyCode::Char('2') => detail.select_tab(DetailTab::Extras),
            KeyCode::Char('3') => {
                detail.select_tab(DetailTab::Saves);
                trigger_screenshot = true;
            }
            KeyCode::Char('4') => detail.select_tab(DetailTab::Achievements),
            KeyCode::Char('5') => detail.select_tab(DetailTab::Technical),
            KeyCode::Up | KeyCode::Char('k') if detail.active_tab == DetailTab::Extras => {
                detail.extras_selection_previous();
            }
            KeyCode::Down | KeyCode::Char('j') if detail.active_tab == DetailTab::Extras => {
                detail.extras_selection_next();
            }
            KeyCode::Char(' ') if detail.active_tab == DetailTab::Extras => {
                detail.extras_toggle_current();
            }
            KeyCode::Char('a') | KeyCode::Char('A') if detail.active_tab == DetailTab::Extras => {
                detail.extras_toggle_all();
            }
            KeyCode::Up | KeyCode::Char('k') if detail.active_tab == DetailTab::Saves => {
                detail.save_selection_previous();
                trigger_screenshot = true;
            }
            KeyCode::Down | KeyCode::Char('j') if detail.active_tab == DetailTab::Saves => {
                detail.save_selection_next();
                trigger_screenshot = true;
            }
            KeyCode::Up | KeyCode::Char('k') if detail.active_tab == DetailTab::Achievements => {
                detail.achievement_selection_previous();
            }
            KeyCode::Down | KeyCode::Char('j') if detail.active_tab == DetailTab::Achievements => {
                detail.achievement_selection_next();
            }
            KeyCode::Char('u') if detail.active_tab == DetailTab::Saves => {
                detail.open_save_upload_picker()
            }
            KeyCode::Char('D') if detail.active_tab == DetailTab::Saves => {
                let Some(save) = detail.selected_save().cloned() else {
                    detail.message = Some("No save selected".into());
                    detail.message_clear_at = Some(Instant::now() + Duration::from_secs(3));
                    return Ok(false);
                };
                let rom_id = detail.rom.id;
                let rom = detail.rom.clone();
                let target_dir = match resolve_game_save_dir(&self.config, &rom) {
                    Ok(path) => path,
                    Err(err) => {
                        detail.message = Some(format!(
                            "Save download blocked: {err:#}. Fix save paths in Settings."
                        ));
                        detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                        return Ok(false);
                    }
                };
                detail.message = Some("Downloading save...".into());
                detail.message_clear_at = None;
                let client = self.client.clone();
                let tx = self.save_download_tx.clone();
                tokio::spawn(async move {
                    let result: Result<PathBuf, RommError> = async {
                        let bytes = client.download_save_content(save.id, None, None).await?;
                        tokio::fs::create_dir_all(&target_dir)
                            .await
                            .map_err(|e| RommError::Api(ApiError::Io(e)))?;
                        let filename = if save.file_name.trim().is_empty() {
                            format!("save-{}.sav", save.id)
                        } else {
                            save.file_name.clone()
                        };
                        let target = reserve_unique_save_path(&target_dir, &filename);
                        tokio::fs::write(&target, bytes)
                            .await
                            .map_err(|e| RommError::Api(ApiError::Io(e)))?;
                        Ok(target)
                    }
                    .await;
                    let _ = tx.send(SaveDownloadDone { rom_id, result });
                });
            }
            KeyCode::Enter if detail.active_tab == DetailTab::Extras => {
                if detail.extras_selected_count() == 0 {
                    detail.message = Some("Select at least one item (Space to toggle)".into());
                    detail.message_clear_at = Some(Instant::now() + Duration::from_secs(2));
                } else {
                    let rom = detail.rom.clone();
                    let items = &detail.extras_items;
                    let targets =
                        crate::tui::screens::extras_picker::build_selected_targets_from_items(
                            items,
                            &rom,
                            &self.config.roms_layout,
                            Some(self.config.download_dir.as_str()),
                        );
                    match targets {
                        Ok(targets) => {
                            match self.downloads.start_extras_download(
                                &rom,
                                targets,
                                self.client.clone(),
                                Some(self.config.download_dir.as_str()),
                            ) {
                                Ok(()) => {
                                    detail.message = Some("Extras download started".into());
                                    detail.message_clear_at =
                                        Some(Instant::now() + Duration::from_secs(3));
                                }
                                Err(e) => {
                                    detail.message = Some(format!("Extras: {e:#}"));
                                    detail.message_clear_at =
                                        Some(Instant::now() + Duration::from_secs(5));
                                }
                            }
                        }
                        Err(e) => {
                            detail.message = Some(format!("{e:#}"));
                            detail.message_clear_at = Some(Instant::now() + Duration::from_secs(4));
                        }
                    }
                }
            }
            // Only start a download once per detail view and avoid
            // stacking multiple concurrent downloads for the same ROM.
            KeyCode::Enter if !detail.has_started_download => {
                match self.downloads.start_download(
                    &detail.rom,
                    self.client.clone(),
                    &self.config.roms_layout,
                    Some(self.config.download_dir.as_str()),
                ) {
                    Ok(()) => {
                        detail.has_started_download = true;
                        if has_update_or_dlc_extras(&detail.rom, &detail.other_files) {
                            detail.message = Some(
                                "Updates/DLC available. Switch to Extras tab to download."
                                    .to_string(),
                            );
                            detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                        }
                    }
                    Err(err) => {
                        detail.has_started_download = false;
                        detail.message = Some(format!(
                            "Download blocked: {err}. Fix ROMs directory in settings/setup."
                        ));
                    }
                }
            }
            KeyCode::Char('o') => detail.open_cover(),
            KeyCode::Esc => {
                detail.clear_message();
                let placeholder = self.transient_screen_placeholder();
                let prev = std::mem::replace(&mut self.screen, placeholder);
                if let AppScreen::GameDetail(g) = prev {
                    self.screen = match g.previous {
                        GameDetailPrevious::Library(l) => AppScreen::LibraryBrowse(l),
                        GameDetailPrevious::Search(s) => AppScreen::Search(s),
                    };
                    self.resume_library_rom_load_if_needed("restore_partial_library");
                }
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        if trigger_screenshot {
            self.maybe_start_save_screenshot_load();
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_save_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "romm-tui-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reserve_unique_save_path_reserves_collision_candidates() {
        let dir = temp_save_dir("save-path-reservation");

        let first = reserve_unique_save_path(&dir, "same.sav");
        let second = reserve_unique_save_path(&dir, "same.sav");

        assert_ne!(first, second);
        assert!(first.exists());
        assert!(second.exists());

        let _ = std::fs::remove_dir_all(dir);
    }
}
