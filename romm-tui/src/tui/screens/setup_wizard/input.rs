//! Setup wizard keyboard and paste input.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::tui::path_picker::PathPickerEvent;
use romm_api::config::normalize_romm_origin;
use romm_api::core::download::validate_configured_download_directory;

use super::types::{AuthKind, SetupWizard, Step};

impl SetupWizard {
    fn add_char_url(&mut self, c: char) {
        let pos = self.url_cursor.min(self.url.len());
        self.url.insert(pos, c);
        self.url_cursor = pos + 1;
    }

    fn del_char_url(&mut self) {
        if self.url_cursor > 0 && self.url_cursor <= self.url.len() {
            self.url.remove(self.url_cursor - 1);
            self.url_cursor -= 1;
        }
    }

    fn advance_from_auth_menu(&mut self) {
        self.auth_kind = Self::auth_kind_from_index(self.auth_menu_selected);
        self.step = match self.auth_kind {
            AuthKind::Basic => Step::BasicUser,
            AuthKind::Bearer => Step::Bearer,
            AuthKind::ApiKey => Step::ApiHeader,
            AuthKind::Pairing => {
                self.pairing_cursor = self.pairing_code.len();
                Step::PairingCode
            }
        };
    }

    fn advance_after_auth_credentials(&mut self) {
        self.step = if self.skip_custom_console_paths {
            Step::Summary
        } else {
            Step::CustomConsolePaths
        };
    }

    fn advance_step(&mut self) -> Result<()> {
        self.error = None;
        match self.step {
            Step::Url => {
                if normalize_romm_origin(self.url.trim()).is_empty() {
                    self.error = Some("Enter a valid server URL".to_string());
                    return Ok(());
                }
                self.step = Step::Https;
            }
            Step::Https => {
                self.step = Step::Download;
            }
            Step::Download => {}
            Step::CustomConsolePaths => {
                self.step = Step::Summary;
            }
            Step::AuthMenu => self.advance_from_auth_menu(),
            Step::BasicUser => self.step = Step::BasicPass,
            Step::BasicPass => self.advance_after_auth_credentials(),
            Step::Bearer => self.advance_after_auth_credentials(),
            Step::ApiHeader => self.step = Step::ApiKey,
            Step::ApiKey => self.advance_after_auth_credentials(),
            Step::PairingCode => self.advance_after_auth_credentials(),
            Step::Summary => {}
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> Result<bool> {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }
        if key.code == KeyCode::Esc {
            return Ok(true); // Signal to caller that we should exit/cancel
        }

        if self.testing {
            return Ok(false);
        }

        match self.step {
            Step::Url => match key.code {
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => self.add_char_url(c),
                KeyCode::Backspace => self.del_char_url(),
                KeyCode::Left if self.url_cursor > 0 => {
                    self.url_cursor -= 1;
                }
                KeyCode::Right if self.url_cursor < self.url.len() => {
                    self.url_cursor += 1;
                }
                _ => {}
            },
            Step::Https => match key.code {
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(' ') => self.use_https = !self.use_https,
                _ => {}
            },
            Step::Download => match self.download_picker.handle_key(key) {
                PathPickerEvent::Confirmed(p) => {
                    self.error = None;
                    match validate_configured_download_directory(p.to_string_lossy().as_ref()) {
                        Ok(canonical) => {
                            self.download_picker
                                .set_path_text(canonical.display().to_string());
                            self.step = Step::AuthMenu;
                        }
                        Err(e) => {
                            self.error = Some(format!("{e:#}"));
                        }
                    }
                }
                PathPickerEvent::None => {}
            },
            Step::CustomConsolePaths => {
                if key.code == KeyCode::Enter {
                    let _ = self.advance_step();
                }
            }
            Step::AuthMenu => match key.code {
                KeyCode::Up | KeyCode::Char('k') if self.auth_menu_selected > 0 => {
                    self.auth_menu_selected -= 1;
                }
                KeyCode::Down | KeyCode::Char('j') if self.auth_menu_selected < 3 => {
                    self.auth_menu_selected += 1;
                }
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                _ => {}
            },
            Step::BasicUser => match key.code {
                KeyCode::Tab => self.step = Step::BasicPass,
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    let pos = self.user_cursor.min(self.username.len());
                    self.username.insert(pos, c);
                    self.user_cursor = pos + 1;
                }
                KeyCode::Backspace
                    if self.user_cursor > 0 && self.user_cursor <= self.username.len() =>
                {
                    self.username.remove(self.user_cursor - 1);
                    self.user_cursor -= 1;
                }
                KeyCode::Left if self.user_cursor > 0 => {
                    self.user_cursor -= 1;
                }
                KeyCode::Right if self.user_cursor < self.username.len() => {
                    self.user_cursor += 1;
                }
                _ => {}
            },
            Step::BasicPass => match key.code {
                KeyCode::Tab => self.step = Step::BasicUser,
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    self.reuse_keyring_password = false;
                    self.password.push(c);
                }
                KeyCode::Backspace => {
                    self.password.pop();
                }
                _ => {}
            },
            Step::Bearer => match key.code {
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    self.reuse_keyring_bearer = false;
                    let pos = self.bearer_cursor.min(self.bearer_token.len());
                    self.bearer_token.insert(pos, c);
                    self.bearer_cursor = pos + 1;
                }
                KeyCode::Backspace
                    if self.bearer_cursor > 0 && self.bearer_cursor <= self.bearer_token.len() =>
                {
                    self.bearer_token.remove(self.bearer_cursor - 1);
                    self.bearer_cursor -= 1;
                }
                KeyCode::Left if self.bearer_cursor > 0 => {
                    self.bearer_cursor -= 1;
                }
                KeyCode::Right if self.bearer_cursor < self.bearer_token.len() => {
                    self.bearer_cursor += 1;
                }
                _ => {}
            },
            Step::PairingCode => match key.code {
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    let pos = self.pairing_cursor.min(self.pairing_code.len());
                    self.pairing_code.insert(pos, c);
                    self.pairing_cursor = pos + 1;
                }
                KeyCode::Backspace
                    if self.pairing_cursor > 0
                        && self.pairing_cursor <= self.pairing_code.len() =>
                {
                    self.pairing_code.remove(self.pairing_cursor - 1);
                    self.pairing_cursor -= 1;
                }
                KeyCode::Left if self.pairing_cursor > 0 => {
                    self.pairing_cursor -= 1;
                }
                KeyCode::Right if self.pairing_cursor < self.pairing_code.len() => {
                    self.pairing_cursor += 1;
                }
                _ => {}
            },
            Step::ApiHeader => match key.code {
                KeyCode::Tab => self.step = Step::ApiKey,
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    let pos = self.header_cursor.min(self.api_header.len());
                    self.api_header.insert(pos, c);
                    self.header_cursor = pos + 1;
                }
                KeyCode::Backspace
                    if self.header_cursor > 0 && self.header_cursor <= self.api_header.len() =>
                {
                    self.api_header.remove(self.header_cursor - 1);
                    self.header_cursor -= 1;
                }
                KeyCode::Left if self.header_cursor > 0 => {
                    self.header_cursor -= 1;
                }
                KeyCode::Right if self.header_cursor < self.api_header.len() => {
                    self.header_cursor += 1;
                }
                _ => {}
            },
            Step::ApiKey => match key.code {
                KeyCode::Tab => self.step = Step::ApiHeader,
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    self.reuse_keyring_api_key = false;
                    let pos = self.api_key_cursor.min(self.api_key.len());
                    self.api_key.insert(pos, c);
                    self.api_key_cursor = pos + 1;
                }
                KeyCode::Backspace
                    if self.api_key_cursor > 0 && self.api_key_cursor <= self.api_key.len() =>
                {
                    self.api_key.remove(self.api_key_cursor - 1);
                    self.api_key_cursor -= 1;
                }
                KeyCode::Left if self.api_key_cursor > 0 => {
                    self.api_key_cursor -= 1;
                }
                KeyCode::Right if self.api_key_cursor < self.api_key.len() => {
                    self.api_key_cursor += 1;
                }
                _ => {}
            },
            Step::Summary => {
                if key.code == KeyCode::Enter {
                    self.testing = true;
                    self.error = None;
                    // The caller handles the actual async try_connect_and_persist call
                    // when they see testing = true.
                }
            }
        }
        Ok(false)
    }

    pub fn handle_paste(&mut self, text: &str) {
        // Remove any newlines or carriage returns that might break single-line fields
        let clean_text = text.replace(['\n', '\r'], "");
        if clean_text.is_empty() {
            return;
        }

        match self.step {
            Step::Url => {
                let pos = self.url_cursor.min(self.url.len());
                self.url.insert_str(pos, &clean_text);
                self.url_cursor += clean_text.len();
            }
            Step::BasicUser => {
                let pos = self.user_cursor.min(self.username.len());
                self.username.insert_str(pos, &clean_text);
                self.user_cursor += clean_text.len();
            }
            Step::BasicPass => {
                self.reuse_keyring_password = false;
                self.password.push_str(&clean_text);
            }
            Step::Bearer => {
                self.reuse_keyring_bearer = false;
                let pos = self.bearer_cursor.min(self.bearer_token.len());
                self.bearer_token.insert_str(pos, &clean_text);
                self.bearer_cursor += clean_text.len();
            }
            Step::PairingCode => {
                let pos = self.pairing_cursor.min(self.pairing_code.len());
                self.pairing_code.insert_str(pos, &clean_text);
                self.pairing_cursor += clean_text.len();
            }
            Step::ApiHeader => {
                let pos = self.header_cursor.min(self.api_header.len());
                self.api_header.insert_str(pos, &clean_text);
                self.header_cursor += clean_text.len();
            }
            Step::ApiKey => {
                self.reuse_keyring_api_key = false;
                let pos = self.api_key_cursor.min(self.api_key.len());
                self.api_key.insert_str(pos, &clean_text);
                self.api_key_cursor += clean_text.len();
            }
            _ => {}
        }
    }
}
