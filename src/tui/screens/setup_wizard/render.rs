//! Setup wizard rendering.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::config::normalize_romm_origin;

use super::layout::{wizard_footer_text, wizard_layout};
use super::types::{AuthKind, SetupWizard, Step};

impl SetupWizard {
    pub fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let title = match self.step {
            Step::Url => "Step 1/6 — RomM server URL",
            Step::Https => "Step 2/6 — Secure connection",
            Step::Download => "Step 3/6 — ROMs directory",
            Step::CustomConsolePaths => "Step 4/6 — Custom console paths",
            Step::AuthMenu => "Step 5/6 — Authentication",
            Step::BasicUser | Step::BasicPass => "Step 6/6 — Basic auth",
            Step::Bearer => "Step 6/6 — API Token",
            Step::ApiHeader | Step::ApiKey => "Step 6/6 — API key",
            Step::PairingCode => "Step 6/6 — Pair with Web UI",
            Step::Summary => "Review & connect",
        };

        let main = wizard_layout(area, self.step);

        match self.step {
            Step::Url => {
                let intro = Text::from(vec![
                    Line::from("First-time setup: point the CLI at your RomM server."),
                    Line::from(Span::styled(
                        "Example: https://romm.example.com or http://192.168.1.10:8080",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(Span::styled(
                        "Same origin as in your browser (no trailing /api).",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]);
                f.render_widget(Paragraph::new(intro), main[0]);
            }
            step => {
                let hint_top = match step {
                    Step::Https => "HTTPS ensures your credentials are encrypted in transit. Only disable if necessary.",
                    Step::Download => "Choose a directory to save ROMs. Make sure you have write permissions.",
                    Step::CustomConsolePaths => "Consoles on other drives can use custom paths. Map them in Settings → ROMs → Console paths after setup.",
                    Step::AuthMenu => "Select how you authenticate with the RomM server.",
                    Step::BasicUser | Step::BasicPass => "Enter the exact same username and password you use to log into the RomM web UI.",
                    Step::Bearer => "To get an API token, go to the RomM web UI -> client API Tokens -> generate a new token.",
                    Step::PairingCode => "Login to RomM in your browser, go to your profile menu -> Client API Tokens, and create a new token.",
                    Step::ApiHeader | Step::ApiKey => "Use this only if you have a custom reverse proxy setup requiring specific headers (e.g., X-Api-Key). Otherwise, use API Token.",
                    Step::Summary => "Review your configuration before testing the connection.",
                    Step::Url => "",
                };
                let p = Paragraph::new(hint_top).style(Style::default().fg(Color::DarkGray));
                f.render_widget(p, main[0]);
            }
        }

        match self.step {
            Step::Url => {
                let line = format!(
                    "{}▏",
                    self.url.chars().take(self.url_cursor).collect::<String>()
                );
                let rest: String = self.url.chars().skip(self.url_cursor).collect();
                let text = format!("{line}{rest}");
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(text).block(block);
                f.render_widget(p, main[1]);
            }
            Step::Https => {
                let text = if self.use_https {
                    "[X] Use HTTPS (Recommended)"
                } else {
                    "[ ] Use HTTPS (Insecure)"
                };
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(format!("\n  {}\n\n  Space: toggle   Enter: next", text))
                    .block(block);
                f.render_widget(p, main[1]);
            }
            Step::Download => {
                self.download_picker.render(f, main[1], title, "");
            }
            Step::CustomConsolePaths => {
                let body = "By default each console uses a subfolder under your ROMs directory.\n\nConsoles on other drives (e.g. Switch on D:, NES on E:) can use custom paths.\nMap them in Settings → ROMs → Console paths after setup.\n\nEnter: next";
                let block = Block::default().title(title).borders(Borders::ALL);
                f.render_widget(Paragraph::new(body).block(block), main[1]);
            }
            Step::AuthMenu => {
                let items: Vec<ListItem> = Self::auth_labels()
                    .iter()
                    .map(|s| ListItem::new(*s))
                    .collect();
                let mut state = ListState::default();
                state.select(Some(self.auth_menu_selected));
                let list = List::new(items)
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, main[1], &mut state);
            }
            Step::BasicUser | Step::BasicPass => {
                let user_line = if self.step == Step::BasicUser {
                    format!(
                        "{}▏{}",
                        self.username
                            .chars()
                            .take(self.user_cursor)
                            .collect::<String>(),
                        self.username
                            .chars()
                            .skip(self.user_cursor)
                            .collect::<String>()
                    )
                } else {
                    self.username.clone()
                };
                let pass_display: String = "•".repeat(self.password.len());
                let kr_hint = if self.step == Step::BasicPass
                    && self.password.is_empty()
                    && self.reuse_keyring_password
                {
                    "\n\n(stored in OS keyring — leave blank to keep, or type a new password)"
                } else {
                    ""
                };
                let block = Block::default().title(title).borders(Borders::ALL);
                let body = format!(
                    "Username\n{user_line}\n\nPassword (hidden)\n{pass_display}{kr_hint}\n\nTab: switch field"
                );
                let p = Paragraph::new(body).block(block);
                f.render_widget(p, main[1]);
            }
            Step::Bearer => {
                let line = format!(
                    "{}▏{}",
                    self.bearer_token
                        .chars()
                        .take(self.bearer_cursor)
                        .collect::<String>(),
                    self.bearer_token
                        .chars()
                        .skip(self.bearer_cursor)
                        .collect::<String>()
                );
                let mut bearer_text = Text::from(vec![
                    Line::from("API Token"),
                    Line::from(""),
                    Line::from(line),
                ]);
                if self.bearer_token.is_empty() && self.reuse_keyring_bearer {
                    bearer_text.push_line(Line::from(""));
                    bearer_text.push_line(Line::from(Span::styled(
                        "Token stored in OS keyring — leave blank to keep, or type a new token.",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(bearer_text).block(block);
                f.render_widget(p, main[1]);
            }
            Step::PairingCode => {
                let line = format!(
                    "{}▏{}",
                    self.pairing_code
                        .chars()
                        .take(self.pairing_cursor)
                        .collect::<String>(),
                    self.pairing_code
                        .chars()
                        .skip(self.pairing_cursor)
                        .collect::<String>()
                );
                let body = format!("Enter the 8-character code provided.\n\n{line}");
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(body).block(block);
                f.render_widget(p, main[1]);
            }
            Step::ApiHeader | Step::ApiKey => {
                let header_line = if self.step == Step::ApiHeader {
                    format!(
                        "{}▏{}",
                        self.api_header
                            .chars()
                            .take(self.header_cursor)
                            .collect::<String>(),
                        self.api_header
                            .chars()
                            .skip(self.header_cursor)
                            .collect::<String>()
                    )
                } else {
                    self.api_header.clone()
                };
                let key_line = "•".repeat(self.api_key.len());
                let kr_hint = if self.step == Step::ApiKey
                    && self.api_key.is_empty()
                    && self.reuse_keyring_api_key
                {
                    "\n\n(stored in OS keyring — leave blank to keep, or type a new key)"
                } else {
                    ""
                };
                let body = format!(
                    "Header name\n{header_line}\n\nKey (hidden)\n{key_line}{kr_hint}\n\nTab: switch field"
                );
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(body).block(block);
                f.render_widget(p, main[1]);
            }
            Step::Summary => {
                let url_line = normalize_romm_origin(self.url.trim());
                let auth_desc = match self.auth_kind {
                    AuthKind::Basic => "Basic",
                    AuthKind::Bearer => "API Token",
                    AuthKind::ApiKey => "API key header",
                    AuthKind::Pairing => {
                        if self.pairing_code.trim().is_empty() {
                            "Pair with Web UI (no code yet)"
                        } else {
                            "Pair with Web UI (code entered)"
                        }
                    }
                };
                let mut lines = vec![
                    format!("Server: {url_line}"),
                    format!("ROMs Dir: {}", self.download_picker.path_trimmed()),
                    "Layout: base subfolder per console (custom paths in Settings → ROMs)"
                        .to_string(),
                    format!("Use HTTPS: {}", if self.use_https { "Yes" } else { "No" }),
                    format!("Auth: {auth_desc}"),
                    String::new(),
                ];
                if self.testing {
                    lines.push("Connecting to server…".to_string());
                } else if let Some(ref e) = self.error {
                    lines.push(format!("Last error: {e}"));
                } else {
                    lines.push("Enter: test connection and save   Esc: quit".to_string());
                }
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(lines.join("\n")).block(block);
                f.render_widget(p, main[1]);
            }
        }

        let footer_keys = match self.step {
            Step::Url => "Enter: next   Backspace: delete   Esc: quit",
            Step::Https => "Space: toggle   Enter: next   Esc: quit",
            Step::Download => "Ctrl+Enter: next (creates path)   ↑ list top: path bar   ↓/↑: list focus   Tab: path/list   Esc: quit",
            Step::CustomConsolePaths => "Enter: next   Esc: quit",
            Step::AuthMenu => "↑/↓: choose   Enter: next   Esc: quit",
            Step::BasicUser | Step::BasicPass => {
                "Type text   Tab: switch field   Enter: next step   Esc: quit"
            }
            Step::Bearer => "Enter: next step   Esc: quit",
            Step::PairingCode => "Enter: next step   Esc: quit",
            Step::ApiHeader | Step::ApiKey => "Tab: switch field   Enter: next step   Esc: quit",
            Step::Summary => {
                if self.testing {
                    "Please wait…"
                } else {
                    "Enter: connect & save"
                }
            }
        };
        let p = Paragraph::new(wizard_footer_text(footer_keys))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, main[2]);
    }

    pub fn cursor_pos(&self, area: ratatui::layout::Rect) -> Option<(u16, u16)> {
        let main = wizard_layout(area, self.step);
        let inner = main[1];
        match self.step {
            Step::Url => {
                let x = inner.x + 1 + self.url_cursor.min(self.url.len()) as u16;
                Some((x, inner.y + 1))
            }
            Step::Download => self
                .download_picker
                .cursor_position(inner, "Step 3/6 — ROMs directory"),
            Step::Bearer => {
                let x = inner.x + 1 + self.bearer_cursor.min(self.bearer_token.len()) as u16;
                Some((x, inner.y + 1))
            }
            Step::PairingCode => {
                let x = inner.x + 1 + self.pairing_cursor.min(self.pairing_code.len()) as u16;
                Some((x, inner.y + 3))
            }
            Step::BasicUser => {
                let x = inner.x + 1 + self.user_cursor.min(self.username.len()) as u16;
                Some((x, inner.y + 2))
            }
            Step::BasicPass => {
                let x = inner.x + 1 + "•".repeat(self.password.len()).len() as u16;
                Some((x, inner.y + 6))
            }
            Step::ApiHeader => {
                let x = inner.x + 1 + self.header_cursor.min(self.api_header.len()) as u16;
                Some((x, inner.y + 2))
            }
            Step::ApiKey => {
                let x = inner.x + 1 + self.api_key_cursor.min(self.api_key.len()) as u16;
                Some((x, inner.y + 6))
            }
            Step::Https | Step::CustomConsolePaths | Step::AuthMenu | Step::Summary => None,
        }
    }
}
