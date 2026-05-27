//! Game detail and extras picker key handlers.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::resolve_game_save_dir;
use crate::core::extras::has_update_or_dlc_extras;

use super::super::background::types::{SaveDownloadDone, SaveUploadDone};
use super::super::{App, AppScreen};
use crate::tui::screens::{ExtrasPickerScreen, GameDetailPrevious, MainMenuScreen};

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

fn unique_save_path(dir: &Path, file_name: &str) -> PathBuf {
    let safe_name = safe_path_segment(file_name);
    let base = Path::new(&safe_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("save");
    let ext = Path::new(&safe_name).extension().and_then(|s| s.to_str());
    let mut candidate = dir.join(&safe_name);
    let mut n = 1u32;
    while candidate.exists() {
        let name = match ext {
            Some(ext) if !ext.is_empty() => format!("{base}-{n}.{ext}"),
            _ => format!("{base}-{n}"),
        };
        candidate = dir.join(name);
        n += 1;
    }
    candidate
}

impl App {
    pub(in crate::tui::app) fn handle_game_detail(&mut self, key: &KeyEvent) -> Result<bool> {
        use crate::tui::path_picker::PathPickerEvent;
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
                            .map_err(|e| format!("{e:#}"));
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
                            crate::core::download::DownloadStatus::Done
                                | crate::core::download::DownloadStatus::SkippedAlreadyExists
                                | crate::core::download::DownloadStatus::Cancelled
                                | crate::core::download::DownloadStatus::FinalizeFailed(_)
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

        let wants_extras = matches!(key.code, KeyCode::Char('e') | KeyCode::Char('E'))
            || (key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::SHIFT));
        if wants_extras {
            if !detail.has_any_extras() {
                detail.message = Some("No extras available for this ROM".to_string());
                detail.message_clear_at = Some(Instant::now() + Duration::from_secs(3));
                return Ok(false);
            }
            let prev =
                std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
            if let AppScreen::GameDetail(g) = prev {
                self.screen = AppScreen::ExtrasPicker(Box::new(ExtrasPickerScreen::new(
                    g,
                    &self.config.extras_defaults,
                )));
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => detail.save_selection_previous(),
            KeyCode::Down | KeyCode::Char('j') => detail.save_selection_next(),
            KeyCode::Char('u') => detail.open_save_upload_picker(),
            KeyCode::Char('D') => {
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
                    let result = async {
                        let bytes = client.download_save_content(save.id, None, None).await?;
                        tokio::fs::create_dir_all(&target_dir).await?;
                        let filename = if save.file_name.trim().is_empty() {
                            format!("save-{}.sav", save.id)
                        } else {
                            save.file_name.clone()
                        };
                        let target = unique_save_path(&target_dir, &filename);
                        tokio::fs::write(&target, bytes).await?;
                        Ok::<PathBuf, anyhow::Error>(target)
                    }
                    .await
                    .map_err(|e| format!("{e:#}"));
                    let _ = tx.send(SaveDownloadDone { rom_id, result });
                });
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
                                "Updates/DLC available. Press e to download extras.".to_string(),
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
            KeyCode::Char('m') => detail.toggle_technical(),
            KeyCode::Esc => {
                detail.clear_message();
                let prev =
                    std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
                if let AppScreen::GameDetail(g) = prev {
                    self.screen = match g.previous {
                        GameDetailPrevious::Library(l) => AppScreen::LibraryBrowse(*l),
                        GameDetailPrevious::Search(s) => AppScreen::Search(s),
                    };
                }
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    pub(in crate::tui::app) fn handle_extras_picker(&mut self, key: &KeyEvent) -> Result<bool> {
        let picker = match &mut self.screen {
            AppScreen::ExtrasPicker(p) => p,
            _ => return Ok(false),
        };
        picker.tick_message();

        match key.code {
            KeyCode::Esc => {
                let prev =
                    std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
                if let AppScreen::ExtrasPicker(p) = prev {
                    self.screen = AppScreen::GameDetail(p.previous);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => picker.move_up(),
            KeyCode::Down | KeyCode::Char('j') => picker.move_down(),
            KeyCode::Char(' ') => picker.toggle_current(),
            KeyCode::Char('a') | KeyCode::Char('A') => picker.toggle_all(),
            KeyCode::Enter => {
                if picker.selected_count() == 0 {
                    picker.show_message(
                        "Select at least one item (Space to toggle)",
                        Duration::from_secs(2),
                    );
                    return Ok(false);
                }
                let targets = match picker.build_selected_targets(
                    &self.config.roms_layout,
                    Some(self.config.download_dir.as_str()),
                ) {
                    Ok(t) => t,
                    Err(e) => {
                        picker.show_message(format!("{e:#}"), Duration::from_secs(4));
                        return Ok(false);
                    }
                };
                let rom = picker.rom.clone();
                let prev =
                    std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
                if let AppScreen::ExtrasPicker(p) = prev {
                    match self.downloads.start_extras_download(
                        &rom,
                        targets,
                        self.client.clone(),
                        Some(self.config.download_dir.as_str()),
                    ) {
                        Ok(()) => {
                            self.screen = AppScreen::GameDetail(p.previous);
                        }
                        Err(e) => {
                            let mut detail = *p.previous;
                            detail.message = Some(format!("Extras: {e:#}"));
                            detail.message_clear_at = Some(Instant::now() + Duration::from_secs(5));
                            self.screen = AppScreen::GameDetail(Box::new(detail));
                        }
                    }
                }
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }
}
