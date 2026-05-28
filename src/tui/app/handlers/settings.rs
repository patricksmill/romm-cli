//! Settings screen key handler.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::client::RommClient;
use crate::config::{auth_for_persist_merge, normalize_romm_origin, Config, ExtrasDefaults};
use crate::endpoints::device::ListDevices;
use crate::endpoints::platforms::ListPlatforms;
use crate::endpoints::sync::TriggerPushPull;

use super::super::background::types::{DeviceListDone, PlatformListDone, SyncPushPullDone};
use super::super::{App, AppScreen};
use crate::tui::screens::settings::{ConsolePathKind, SettingsRow, SettingsTab};
use crate::tui::screens::setup_wizard::SetupWizard;
use crate::tui::screens::MainMenuScreen;
use crate::tui::theme::{resolve_theme_or_default, MessageTone};

impl App {
    async fn refresh_settings_server_version(&mut self) -> Result<()> {
        let (base_url, download_dir, use_https, verbose, auth) = {
            let settings = match &self.screen {
                AppScreen::Settings(s) => s,
                _ => return Ok(()),
            };
            let mut base_url = normalize_romm_origin(settings.base_url.trim());
            if settings.use_https && base_url.starts_with("http://") {
                base_url = base_url.replace("http://", "https://");
            }
            if !settings.use_https && base_url.starts_with("https://") {
                base_url = base_url.replace("https://", "http://");
            }
            (
                base_url,
                settings.download_dir.clone(),
                settings.use_https,
                self.client.verbose(),
                self.config.auth.clone(),
            )
        };
        let cfg = Config {
            base_url,
            download_dir,
            use_https,
            auth,
            extras_defaults: self.config.extras_defaults.clone(),
            save_sync: self.config.save_sync.clone(),
            roms_layout: self.config.roms_layout.clone(),
            theme: self.config.theme.clone(),
        };
        let client = match RommClient::new(&cfg, verbose) {
            Ok(c) => c,
            Err(_) => {
                if let AppScreen::Settings(s) = &mut self.screen {
                    s.server_version = "unavailable (invalid URL or client error)".to_string();
                    self.server_version = None;
                }
                return Ok(());
            }
        };
        let ver = client.rom_server_version_from_heartbeat().await;
        if let AppScreen::Settings(s) = &mut self.screen {
            match ver {
                Some(v) => {
                    s.server_version = v.clone();
                    self.server_version = Some(v);
                }
                None => {
                    s.server_version = "unavailable (heartbeat failed)".to_string();
                    self.server_version = None;
                }
            }
        }
        Ok(())
    }

    pub(in crate::tui::app) async fn handle_settings(&mut self, key: &KeyEvent) -> Result<bool> {
        use crate::core::download::validate_configured_download_directory;
        use crate::tui::path_picker::PathPickerEvent;

        let settings = match &mut self.screen {
            AppScreen::Settings(s) => s,
            _ => return Ok(false),
        };

        if let Some((kind, ref mut picker)) = settings.path_picker {
            if key.code == KeyCode::Esc {
                settings.path_picker = None;
                return Ok(false);
            }
            match picker.handle_key(key) {
                PathPickerEvent::Confirmed(p) => {
                    match validate_configured_download_directory(p.to_string_lossy().as_ref()) {
                        Ok(canonical) => {
                            if kind == crate::tui::screens::settings::SettingsPickerKind::RomsDir {
                                settings.download_dir = canonical.display().to_string();
                                settings.message = Some((
                                    "ROMs directory updated (press S to save)".to_string(),
                                    MessageTone::Success,
                                ));
                            } else {
                                settings.save_dir = canonical.display().to_string();
                                settings.message = Some((
                                    "Save directory updated (press S to save)".to_string(),
                                    MessageTone::Success,
                                ));
                            }
                            settings.path_picker = None;
                        }
                        Err(e) => {
                            settings.message =
                                Some((format!("Invalid ROMs directory: {e:#}"), MessageTone::Error));
                        }
                    }
                }
                PathPickerEvent::None => {}
            }
            return Ok(false);
        }

        if let Some((platform_id, ref mut picker)) = settings.console_path_picker {
            if key.code == KeyCode::Esc {
                settings.console_path_picker = None;
                return Ok(false);
            }
            match picker.handle_key(key) {
                PathPickerEvent::Confirmed(p) => {
                    match validate_configured_download_directory(p.to_string_lossy().as_ref()) {
                        Ok(canonical) => {
                            settings
                                .confirm_console_path(platform_id, canonical.display().to_string());
                        }
                        Err(e) => {
                            settings.message =
                                Some((format!("Invalid console directory: {e:#}"), MessageTone::Error));
                        }
                    }
                }
                PathPickerEvent::None => {}
            }
            return Ok(false);
        }

        if settings.console_picker_open {
            match key.code {
                KeyCode::Esc => {
                    settings.console_picker_open = false;
                    settings.active_console_kind = None;
                }
                KeyCode::Up | KeyCode::Char('k') => settings.console_previous(),
                KeyCode::Down | KeyCode::Char('j') => settings.console_next(),
                KeyCode::Enter => settings.open_console_path_picker(),
                KeyCode::Delete | KeyCode::Backspace => {
                    if let Some(platform) = settings
                        .console_platforms
                        .get(settings.console_selected_index)
                    {
                        settings.clear_console_path(platform.id);
                    }
                }
                _ => {}
            }
            return Ok(false);
        }

        if settings.device_picker_open {
            match key.code {
                KeyCode::Esc => {
                    settings.device_picker_open = false;
                    settings.device_picker_loading = false;
                }
                KeyCode::Up | KeyCode::Char('k') => settings.device_previous(),
                KeyCode::Down | KeyCode::Char('j') => settings.device_next(),
                KeyCode::Enter => settings.confirm_device(),
                _ => {}
            }
            return Ok(false);
        }

        if settings.confirm.is_some() {
            match key.code {
                KeyCode::Enter => match settings.confirm.take().unwrap() {
                    crate::tui::screens::settings::SettingsConfirm::Reset => {
                        let _ = crate::config::reset_all_settings();
                        settings.message = Some((
                            "Settings deleted. Please restart romm-cli.".to_string(),
                            MessageTone::Warning,
                        ));
                    }
                    crate::tui::screens::settings::SettingsConfirm::ClearCache => {
                        match crate::core::cache::RomCache::clear_file() {
                            Ok(true) => {
                                self.rom_cache = crate::core::cache::RomCache::load();
                                settings.message =
                                    Some(("ROM cache cleared.".to_string(), MessageTone::Success));
                            }
                            Ok(false) => {
                                settings.message = Some((
                                    "ROM cache file does not exist.".to_string(),
                                    MessageTone::Warning,
                                ));
                            }
                            Err(e) => {
                                settings.message =
                                    Some((format!("Failed to clear cache: {e}"), MessageTone::Error));
                            }
                        }
                    }
                },
                KeyCode::Esc => {
                    settings.confirm = None;
                }
                _ => {}
            }
            return Ok(false);
        }

        if settings.editing {
            match key.code {
                KeyCode::Enter => {
                    let row = settings.selected_row();
                    settings.save_edit();
                    if row == SettingsRow::BaseUrl {
                        self.refresh_settings_server_version().await?;
                    }
                }
                KeyCode::Esc => settings.cancel_edit(),
                KeyCode::Backspace => settings.delete_char(),
                KeyCode::Left => settings.move_cursor_left(),
                KeyCode::Right => settings.move_cursor_right(),
                KeyCode::Char(c) => settings.add_char(c),
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => settings.previous(),
            KeyCode::Down | KeyCode::Char('j') => settings.next(),
            KeyCode::Right | KeyCode::Char('l') => {
                if settings.selected_tab == SettingsTab::Appearance
                    && settings.selected_row() == SettingsRow::Theme
                {
                    settings.cycle_theme_next();
                    self.theme = resolve_theme_or_default(&settings.theme_id);
                    self.config.theme = settings.theme_id.clone();
                } else {
                    settings.next_tab();
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if settings.selected_tab == SettingsTab::Appearance
                    && settings.selected_row() == SettingsRow::Theme
                {
                    settings.cycle_theme_prev();
                    self.theme = resolve_theme_or_default(&settings.theme_id);
                    self.config.theme = settings.theme_id.clone();
                } else {
                    settings.previous_tab();
                }
            }
            KeyCode::Tab => settings.next_tab(),
            KeyCode::BackTab => settings.previous_tab(),
            KeyCode::Enter => {
                let row = settings.selected_row();
                if row == SettingsRow::Auth {
                    self.screen =
                        AppScreen::SetupWizard(Box::new(SetupWizard::new_auth_only(&self.config)));
                } else if row == SettingsRow::ConsolePaths {
                    settings.open_console_picker(ConsolePathKind::Roms);
                    let client = self.client.clone();
                    let tx = self.platform_list_tx.clone();
                    tokio::spawn(async move {
                        let result = client
                            .call(&ListPlatforms)
                            .await
                            .map_err(|e| format!("{e:#}"));
                        let _ = tx.send(PlatformListDone { result });
                    });
                } else if row == SettingsRow::SaveConsolePaths {
                    settings.open_console_picker(ConsolePathKind::Saves);
                    let client = self.client.clone();
                    let tx = self.platform_list_tx.clone();
                    tokio::spawn(async move {
                        let result = client
                            .call(&ListPlatforms)
                            .await
                            .map_err(|e| format!("{e:#}"));
                        let _ = tx.send(PlatformListDone { result });
                    });
                } else if row == SettingsRow::SyncDevice {
                    if !settings.save_sync_supported() {
                        settings.set_save_sync_unsupported_message();
                        return Ok(false);
                    }
                    settings.enter_edit();
                    let client = self.client.clone();
                    let tx = self.device_list_tx.clone();
                    tokio::spawn(async move {
                        let result = client
                            .call(&ListDevices)
                            .await
                            .map_err(|e| format!("{e:#}"));
                        let _ = tx.send(DeviceListDone { result });
                    });
                } else if row == SettingsRow::SyncNow {
                    if !settings.save_sync_supported() {
                        settings.set_save_sync_unsupported_message();
                        return Ok(false);
                    }
                    if settings.sync_inflight {
                        return Ok(false);
                    }
                    let Some(device_id) = settings.sync_device_id.clone() else {
                        settings.message =
                            Some(("Choose a Sync Device first".to_string(), MessageTone::Warning));
                        return Ok(false);
                    };
                    settings.sync_inflight = true;
                    settings.message =
                        Some(("Sync Saves Now running...".to_string(), MessageTone::Warning));
                    let client = self.client.clone();
                    let tx = self.sync_push_pull_tx.clone();
                    tokio::spawn(async move {
                        let result = client
                            .call(&TriggerPushPull { device_id })
                            .await
                            .map_err(|e| format!("{e:#}"));
                        let _ = tx.send(SyncPushPullDone { result });
                    });
                } else {
                    let toggle_https = row == SettingsRow::UseHttps;
                    settings.enter_edit();
                    if toggle_https {
                        self.refresh_settings_server_version().await?;
                    }
                }
            }
            KeyCode::Char('s' | 'S') => {
                // Save to disk (accept both cases; footer shows "S:")
                use crate::config::persist_user_config;
                let auth = auth_for_persist_merge(self.config.auth.clone());
                let cfg = Config {
                    base_url: settings.base_url.clone(),
                    download_dir: settings.download_dir.clone(),
                    use_https: settings.use_https,
                    auth,
                    extras_defaults: ExtrasDefaults {
                        include_related_roms: settings.extras_include_related_roms,
                        include_cover: settings.extras_include_cover,
                        include_manual: settings.extras_include_manual,
                    },
                    save_sync: settings.save_sync_config(),
                    roms_layout: settings.roms_layout_config(),
                    theme: settings.theme_id.clone(),
                };
                if let Err(e) = persist_user_config(&cfg) {
                    settings.message = Some((format!("Error saving: {e}"), MessageTone::Error));
                } else {
                    settings.message = Some(("Saved to config.json".to_string(), MessageTone::Success));
                    // Update app state
                    self.config.base_url = cfg.base_url.clone();
                    self.config.download_dir = cfg.download_dir.clone();
                    self.config.use_https = cfg.use_https;
                    self.config.extras_defaults = cfg.extras_defaults.clone();
                    self.config.save_sync = cfg.save_sync.clone();
                    self.config.roms_layout = cfg.roms_layout.clone();
                    self.config.theme = cfg.theme.clone();
                    // Re-create client to pick up new base URL
                    if let Ok(new_client) = RommClient::new(&self.config, self.client.verbose()) {
                        self.client = new_client;
                    }
                }
            }
            KeyCode::Esc => self.screen = AppScreen::MainMenu(MainMenuScreen::new()),
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }
}
