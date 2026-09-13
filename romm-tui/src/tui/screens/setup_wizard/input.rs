//! Setup wizard keyboard and paste input.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::tui::path_picker::PathPickerEvent;
use crate::tui::text_cursor;
use romm_api::config::normalize_romm_origin;
use romm_api::core::download::validate_configured_download_directory;

use super::types::{AuthKind, SetupWizard, Step};

impl SetupWizard {
    fn add_char_url(&mut self, c: char) {
        text_cursor::insert_char(&mut self.url, &mut self.url_cursor, c);
    }

    fn del_char_url(&mut self) {
        text_cursor::delete_previous_char(&mut self.url, &mut self.url_cursor);
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
                KeyCode::Left => text_cursor::move_left(&self.url, &mut self.url_cursor),
                KeyCode::Right => text_cursor::move_right(&self.url, &mut self.url_cursor),
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
                    text_cursor::insert_char(&mut self.username, &mut self.user_cursor, c);
                }
                KeyCode::Backspace => {
                    text_cursor::delete_previous_char(&mut self.username, &mut self.user_cursor);
                }
                KeyCode::Left => text_cursor::move_left(&self.username, &mut self.user_cursor),
                KeyCode::Right => text_cursor::move_right(&self.username, &mut self.user_cursor),
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
                    text_cursor::insert_char(&mut self.bearer_token, &mut self.bearer_cursor, c);
                }
                KeyCode::Backspace => {
                    text_cursor::delete_previous_char(
                        &mut self.bearer_token,
                        &mut self.bearer_cursor,
                    );
                }
                KeyCode::Left => {
                    text_cursor::move_left(&self.bearer_token, &mut self.bearer_cursor);
                }
                KeyCode::Right => {
                    text_cursor::move_right(&self.bearer_token, &mut self.bearer_cursor);
                }
                _ => {}
            },
            Step::PairingCode => match key.code {
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    text_cursor::insert_char(&mut self.pairing_code, &mut self.pairing_cursor, c);
                }
                KeyCode::Backspace => {
                    text_cursor::delete_previous_char(
                        &mut self.pairing_code,
                        &mut self.pairing_cursor,
                    );
                }
                KeyCode::Left => {
                    text_cursor::move_left(&self.pairing_code, &mut self.pairing_cursor);
                }
                KeyCode::Right => {
                    text_cursor::move_right(&self.pairing_code, &mut self.pairing_cursor);
                }
                _ => {}
            },
            Step::ApiHeader => match key.code {
                KeyCode::Tab => self.step = Step::ApiKey,
                KeyCode::Enter => {
                    let _ = self.advance_step();
                }
                KeyCode::Char(c) => {
                    text_cursor::insert_char(&mut self.api_header, &mut self.header_cursor, c);
                }
                KeyCode::Backspace => {
                    text_cursor::delete_previous_char(
                        &mut self.api_header,
                        &mut self.header_cursor,
                    );
                }
                KeyCode::Left => text_cursor::move_left(&self.api_header, &mut self.header_cursor),
                KeyCode::Right => {
                    text_cursor::move_right(&self.api_header, &mut self.header_cursor);
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
                    text_cursor::insert_char(&mut self.api_key, &mut self.api_key_cursor, c);
                }
                KeyCode::Backspace => {
                    text_cursor::delete_previous_char(&mut self.api_key, &mut self.api_key_cursor);
                }
                KeyCode::Left => text_cursor::move_left(&self.api_key, &mut self.api_key_cursor),
                KeyCode::Right => text_cursor::move_right(&self.api_key, &mut self.api_key_cursor),
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
                text_cursor::insert_str(&mut self.url, &mut self.url_cursor, &clean_text);
            }
            Step::BasicUser => {
                text_cursor::insert_str(&mut self.username, &mut self.user_cursor, &clean_text);
            }
            Step::BasicPass => {
                self.reuse_keyring_password = false;
                self.password.push_str(&clean_text);
            }
            Step::Bearer => {
                self.reuse_keyring_bearer = false;
                text_cursor::insert_str(
                    &mut self.bearer_token,
                    &mut self.bearer_cursor,
                    &clean_text,
                );
            }
            Step::PairingCode => {
                text_cursor::insert_str(
                    &mut self.pairing_code,
                    &mut self.pairing_cursor,
                    &clean_text,
                );
            }
            Step::ApiHeader => {
                text_cursor::insert_str(&mut self.api_header, &mut self.header_cursor, &clean_text);
            }
            Step::ApiKey => {
                self.reuse_keyring_api_key = false;
                text_cursor::insert_str(&mut self.api_key, &mut self.api_key_cursor, &clean_text);
            }
            _ => {}
        }
    }
}
