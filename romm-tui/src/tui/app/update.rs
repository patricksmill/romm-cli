//! Apply semantic [`Action`]s to [`super::App`] state.

use anyhow::Result;

use crate::tui::screens::library_browse::LibrarySubsection;
use romm_api::core::library_scan::ScanCacheInvalidate;

use super::background::types::{RomLoadDone, RomLoadEvent};
use crate::tui::keyboard_help::apply_keyboard_help_scroll;

use super::event::Action;
use super::App;
use super::AppScreen;

impl App {
    /// Returns `true` when the loop should exit ([`Action::Quit`]).
    pub(crate) async fn update(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Quit => return Ok(true),
            Action::DismissGlobalMessage => {
                self.global_error = None;
                self.global_notice = None;
            }
            Action::DismissStartupSplash => {
                self.startup_splash = None;
            }
            Action::ShowKeyboardHelp => {
                self.show_keyboard_help = true;
                self.keyboard_help_scroll = 0;
            }
            Action::HideKeyboardHelp => self.show_keyboard_help = false,
            Action::KeyboardHelpInput(input) => {
                const HELP_SCROLL_PAGE: u16 = 8;
                self.keyboard_help_scroll =
                    apply_keyboard_help_scroll(self.keyboard_help_scroll, input, HELP_SCROLL_PAGE);
            }
            Action::ToggleDownloadOverlay => self.toggle_download_screen(),
            Action::CloseDownloadOverlay => {
                if matches!(self.screen, AppScreen::Download(_)) {
                    let stored = self.screen_before_download.take();
                    self.restore_screen_or_library(stored);
                }
            }
            Action::ToggleSearchOverlay => self.toggle_search_screen(),
            Action::ToggleSettingsOverlay => self.toggle_settings_screen(),
            Action::RescanLibrary(inv) => self.apply_rescan_library(inv),
            Action::ToggleLibraryUploadPrompt => self.apply_toggle_library_upload_prompt(),
            Action::ProcessDeferredRomLoad => self.process_deferred_rom_load(),
            Action::ApplyStartupUpdate => self.apply_startup_update().await,
            Action::StartupUpdatePromptStart => {
                if let Some(ref mut prompt) = self.startup_update_prompt {
                    if !prompt.updating {
                        prompt.updating = true;
                    }
                }
            }
            Action::StartupUpdatePromptOpenChangelog => {
                if let Some(ref prompt) = self.startup_update_prompt {
                    if let Err(err) = crate::update::open_changelog_in_browser() {
                        self.global_error = Some(format!("Could not open changelog: {err:#}"));
                    } else {
                        self.global_notice =
                            Some(format!("Opened changelog: {}", prompt.status.changelog_url));
                    }
                }
            }
            Action::StartupUpdatePromptDismiss => {
                self.startup_update_prompt = None;
            }
            Action::LibraryKey(key) => {
                if self.handle_library_browse(&key).await? {
                    return Ok(true);
                }
            }
            Action::SearchKey(key) => {
                if self.handle_search(&key).await? {
                    return Ok(true);
                }
            }
            Action::SettingsKey(key) => {
                if self.handle_settings(&key).await? {
                    return Ok(true);
                }
            }
            Action::GameDetailKey(key) => {
                if self.handle_game_detail(&key)? {
                    return Ok(true);
                }
            }
            Action::ExtrasPickerKey(key) => {
                if self.handle_extras_picker(&key)? {
                    return Ok(true);
                }
            }
            Action::SetupWizardKey(key) => {
                if self.handle_setup_wizard(&key).await? {
                    return Ok(true);
                }
            }
            Action::Background(bg) => self.apply_background(bg),
        }
        Ok(false)
    }

    fn apply_rescan_library(&mut self, inv: ScanCacheInvalidate) {
        if let AppScreen::LibraryBrowse(ref lib) = self.screen {
            if !lib.any_upload_prompt_open()
                && !self.library_upload_inflight
                && !self.library_scan_inflight
            {
                self.spawn_library_rescan_worker(inv);
            }
        }
    }

    fn apply_toggle_library_upload_prompt(&mut self) {
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            if lib.any_upload_prompt_open() {
                lib.close_upload_prompt();
            } else if !self.library_upload_inflight && !self.library_scan_inflight {
                if lib.subsection == LibrarySubsection::ByConsole {
                    lib.open_upload_prompt();
                } else {
                    lib.set_metadata_footer(Some("Upload requires Consoles view — press t".into()));
                }
            }
        }
    }

    async fn apply_startup_update(&mut self) {
        let Some(ref mut prompt) = self.startup_update_prompt else {
            return;
        };
        if !prompt.updating {
            return;
        }

        if prompt.status.latest_version == "9.9.9-mock" {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            self.global_notice = Some("Mock update successful! (No files were changed)".into());
            self.startup_update_prompt = None;
            return;
        }

        let options = crate::update::ApplyUpdateOptions {
            show_progress: false,
            show_output: false,
            no_confirm: true,
            target_version_tag: Some(prompt.status.release_tag.clone()),
        };
        match crate::update::apply_update(None, options).await {
            Ok(crate::update::ApplyUpdateOutcome::Updated(version)) => {
                self.global_notice = Some(format!(
                    "Updated to {version}. Restart romm-cli to use the new version."
                ));
            }
            Ok(crate::update::ApplyUpdateOutcome::UpToDate(version)) => {
                self.global_notice = Some(format!("Already up to date (`{version}`)."));
            }
            Err(err) => {
                self.global_error = Some(format!("Update failed: {err:#}"));
            }
        }
        self.startup_update_prompt = None;
    }

    fn process_deferred_rom_load(&mut self) {
        let Some((key, req, expected, context, started)) = self.deferred_load_roms.take() else {
            return;
        };

        if let Some(ref k) = key {
            if let Some(cached) = self.rom_cache.get_valid(k, expected) {
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    if super::rom_load::primary_rom_load_result_matches_selection(lib, &key) {
                        lib.set_roms(cached.clone());
                        lib.set_rom_loading(false);
                        tracing::debug!(
                            "rom-list-render context={} latency_ms={} (cache_hit)",
                            context,
                            started.elapsed().as_millis()
                        );
                    } else {
                        lib.set_rom_loading(false);
                        tracing::debug!(
                            "rom-list-render context={} skipped stale cache hit",
                            context
                        );
                    }
                }
                return;
            }
        }

        if started.elapsed() < std::time::Duration::from_millis(250) {
            self.deferred_load_roms = Some((key, req, expected, context, started));
            return;
        }

        let gen = self.rom_load_gen;
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            lib.set_rom_loading(expected > 0);
        }
        if expected == 0 {
            if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                lib.set_rom_loading(false);
            }
            return;
        }

        let Some(r) = req else {
            if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                lib.set_rom_loading(false);
            }
            return;
        };
        let client = self.client.clone();
        let tx = self.rom_load_tx.clone();

        self.rom_load_task = Some(tokio::spawn(async move {
            let mut req = r;
            let mut aggregated: Option<romm_api::types::RomList> = None;

            loop {
                match client.call(&req).await {
                    Ok(mut batch) => {
                        if let Some(ref mut all) = aggregated {
                            if batch.items.is_empty() {
                                break;
                            }
                            all.items.append(&mut batch.items);
                            let _ = tx.send(RomLoadDone {
                                gen,
                                key: key.clone(),
                                expected,
                                event: RomLoadEvent::Batch(all.clone()),
                                context,
                                started,
                            });
                            if all.items.len() as u64 >= all.total {
                                break;
                            }
                            req.offset = Some(all.items.len() as u32);
                        } else {
                            let loaded = batch.items.len() as u64;
                            let total = batch.total;
                            let _ = tx.send(RomLoadDone {
                                gen,
                                key: key.clone(),
                                expected,
                                event: RomLoadEvent::Batch(batch.clone()),
                                context,
                                started,
                            });
                            req.offset = Some(loaded as u32);
                            aggregated = Some(batch);
                            if loaded >= total {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(RomLoadDone {
                            gen,
                            key: key.clone(),
                            expected,
                            event: RomLoadEvent::Failed(format!("{e:#}")),
                            context,
                            started,
                        });
                        return;
                    }
                }
                if let Some(ref all) = aggregated {
                    if all.items.len() >= 20000 {
                        break;
                    }
                }
            }

            let _ = tx.send(RomLoadDone {
                gen,
                key,
                expected,
                event: RomLoadEvent::Complete,
                context,
                started,
            });
        }));
    }
}
