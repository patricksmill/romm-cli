use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::config::Config;

#[derive(PartialEq, Eq)]
pub enum SettingsField {
    BaseUrl,
    DownloadDir,
    UseHttps,
}

/// Interactive settings screen for editing current config.
pub struct SettingsScreen {
    pub base_url: String,
    pub download_dir: String,
    pub use_https: bool,
    pub auth_status: String,
    pub version: String,
    pub server_version: String,
    pub github_url: String,

    pub selected_index: usize,
    pub editing: bool,
    pub edit_buffer: String,
    pub edit_cursor: usize,
    pub message: Option<(String, Color)>,
}

impl SettingsScreen {
    pub fn new(config: &Config, romm_server_version: Option<&str>) -> Self {
        let auth_status = match &config.auth {
            Some(crate::config::AuthConfig::Basic { username, .. }) => {
                format!("Basic (user: {})", username)
            }
            Some(crate::config::AuthConfig::Bearer { .. }) => "API Token".to_string(),
            Some(crate::config::AuthConfig::ApiKey { header, .. }) => {
                format!("API key (header: {})", header)
            }
            None => "None (no API credentials in env/keyring)".to_string(),
        };

        let server_version = romm_server_version
            .map(String::from)
            .unwrap_or_else(|| "unavailable (heartbeat failed)".to_string());

        Self {
            base_url: config.base_url.clone(),
            download_dir: config.download_dir.clone(),
            use_https: config.use_https,
            auth_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            server_version,
            github_url: "https://github.com/patricksmill/romm-cli".to_string(),
            selected_index: 0,
            editing: false,
            edit_buffer: String::new(),
            edit_cursor: 0,
            message: None,
        }
    }

    pub fn next(&mut self) {
        if !self.editing {
            self.selected_index = (self.selected_index + 1) % 4;
        }
    }

    pub fn previous(&mut self) {
        if !self.editing {
            if self.selected_index == 0 {
                self.selected_index = 3;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn enter_edit(&mut self) {
        if self.selected_index == 2 {
            // Toggle HTTPS directly
            self.use_https = !self.use_https;
            if self.use_https && self.base_url.starts_with("http://") {
                self.base_url = self.base_url.replace("http://", "https://");
                self.message = Some(("Upgraded to HTTPS".to_string(), Color::Green));
            }
        } else {
            self.editing = true;
            self.edit_buffer = if self.selected_index == 0 {
                self.base_url.clone()
            } else {
                self.download_dir.clone()
            };
            self.edit_cursor = self.edit_buffer.len();
        }
    }

    pub fn save_edit(&mut self) -> bool {
        if !self.editing {
            return true; // UseHttps toggle is "saved" immediately in memory
        }
        if self.selected_index == 0 {
            self.base_url = self.edit_buffer.trim().to_string();
        } else if self.selected_index == 1 {
            self.download_dir = self.edit_buffer.trim().to_string();
        }
        self.editing = false;
        true
    }

    pub fn cancel_edit(&mut self) {
        self.editing = false;
        self.message = None;
    }

    pub fn add_char(&mut self, c: char) {
        if self.editing {
            self.edit_buffer.insert(self.edit_cursor, c);
            self.edit_cursor += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if self.editing && self.edit_cursor > 0 {
            self.edit_buffer.remove(self.edit_cursor - 1);
            self.edit_cursor -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.editing && self.edit_cursor > 0 {
            self.edit_cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.editing && self.edit_cursor < self.edit_buffer.len() {
            self.edit_cursor += 1;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4), // Header info
                Constraint::Min(10),   // Editable list
                Constraint::Length(3), // Message/Hint
                Constraint::Length(3), // Footer help
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        // -- Header Info --
        let info = [
            format!(
                "romm-cli: v{} | RomM server: {}",
                self.version, self.server_version
            ),
            format!("GitHub:   {}", self.github_url),
            format!("Auth:     {}", self.auth_status),
        ];
        f.render_widget(
            Paragraph::new(info.join("\n")).block(Block::default().borders(Borders::BOTTOM)),
            chunks[0],
        );

        // -- Editable List --
        let items = [
            ListItem::new(format!(
                "Base URL:     {}",
                if self.editing && self.selected_index == 0 {
                    &self.edit_buffer
                } else {
                    &self.base_url
                }
            )),
            ListItem::new(format!(
                "Download Dir: {}",
                if self.editing && self.selected_index == 1 {
                    &self.edit_buffer
                } else {
                    &self.download_dir
                }
            )),
            ListItem::new(format!(
                "Use HTTPS:    {}",
                if self.use_https { "[X] Yes" } else { "[ ] No" }
            )),
            ListItem::new(format!(
                "Auth:         {} (Enter to change)",
                self.auth_status
            )),
        ];

        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Configuration ")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[1], &mut state);

        // -- Message Area --
        if let Some((msg, color)) = &self.message {
            f.render_widget(
                Paragraph::new(msg.as_str()).style(Style::default().fg(*color)),
                chunks[2],
            );
        } else if self.editing {
            f.render_widget(
                Paragraph::new("Editing... Enter: save   Esc: cancel")
                    .style(Style::default().fg(Color::Cyan)),
                chunks[2],
            );
        }

        // -- Footer Help --
        let help = if self.editing {
            "Backspace: delete   Arrows: move cursor   Enter: save   Esc: cancel"
        } else {
            "↑/↓: select   Enter: edit/toggle   S: save to disk   Esc: back"
        };
        f.render_widget(
            Paragraph::new(help).block(Block::default().borders(Borders::ALL)),
            chunks[3],
        );
    }

    pub fn cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        if !self.editing {
            return None;
        }

        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let list_area = chunks[1];
        let y = list_area.y + 1 + self.selected_index as u16;
        // Both branches were identical; keep the constant for cursor alignment.
        let label_len = 17;
        let x = list_area.x + 3 + label_len + self.edit_cursor as u16;

        Some((x, y))
    }
}
