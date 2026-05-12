use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::config::{disk_has_unresolved_keyring_sentinel, Config};
use crate::endpoints::device::DeviceSchema;
use crate::tui::path_picker::{PathPicker, PathPickerMode};

#[derive(PartialEq, Eq)]
pub enum SettingsField {
    BaseUrl,
    DownloadDir,
    UseHttps,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsPickerKind {
    RomsDir,
    SaveDir,
}

#[derive(PartialEq, Eq)]
pub enum SettingsConfirm {
    Reset,
    ClearCache,
}

/// Interactive settings screen for editing current config.
pub struct SettingsScreen {
    pub base_url: String,
    pub download_dir: String,
    pub use_https: bool,
    /// Default: pre-check related ROMs (updates/DLC) in TUI extras picker.
    pub extras_include_related_roms: bool,
    /// Default: pre-check cover in TUI extras picker when available.
    pub extras_include_cover: bool,
    /// Default: pre-check manual in TUI extras picker when available.
    pub extras_include_manual: bool,
    pub auth_status: String,
    pub version: String,
    pub server_version: String,
    pub github_url: String,

    pub selected_index: usize,
    pub editing: bool,
    pub confirm: Option<SettingsConfirm>,
    pub edit_buffer: String,
    pub edit_cursor: usize,
    /// ROMs directory browser (`None` when not choosing a folder).
    pub path_picker: Option<(SettingsPickerKind, PathPicker)>,
    pub save_dir: String,
    pub sync_device_id: Option<String>,
    pub devices: Vec<DeviceSchema>,
    pub device_picker_open: bool,
    pub device_picker_loading: bool,
    pub device_picker_error: Option<String>,
    pub device_selected_index: usize,
    pub sync_inflight: bool,
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
            None => {
                if disk_has_unresolved_keyring_sentinel(config) {
                    "None — disk still references keyring; set API_TOKEN / ROMM_TOKEN_FILE or see docs/troubleshooting-auth.md"
                        .to_string()
                } else {
                    "None (no API credentials in env/keyring)".to_string()
                }
            }
        };

        let server_version = romm_server_version
            .map(String::from)
            .unwrap_or_else(|| "unavailable (heartbeat failed)".to_string());

        Self {
            base_url: config.base_url.clone(),
            download_dir: config.download_dir.clone(),
            save_dir: crate::config::resolved_save_dir(config)
                .display()
                .to_string(),
            sync_device_id: config.save_sync.device_id.clone(),
            use_https: config.use_https,
            extras_include_related_roms: config.extras_defaults.include_related_roms,
            extras_include_cover: config.extras_defaults.include_cover,
            extras_include_manual: config.extras_defaults.include_manual,
            auth_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            server_version,
            github_url: "https://github.com/patricksmill/romm-cli".to_string(),
            selected_index: 0,
            editing: false,
            confirm: None,
            edit_buffer: String::new(),
            edit_cursor: 0,
            path_picker: None,
            devices: Vec::new(),
            device_picker_open: false,
            device_picker_loading: false,
            device_picker_error: None,
            device_selected_index: 0,
            sync_inflight: false,
            message: None,
        }
    }

    const ROW_COUNT: usize = 12;

    pub fn next(&mut self) {
        if !self.editing && self.confirm.is_none() {
            self.selected_index = (self.selected_index + 1) % Self::ROW_COUNT;
        }
    }

    pub fn previous(&mut self) {
        if !self.editing && self.confirm.is_none() {
            if self.selected_index == 0 {
                self.selected_index = Self::ROW_COUNT - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn enter_edit(&mut self) {
        if self.selected_index == 11 {
            self.confirm = Some(SettingsConfirm::Reset);
        } else if self.selected_index == 10 {
            self.confirm = Some(SettingsConfirm::ClearCache);
        } else if self.selected_index == 8 {
            self.device_picker_open = true;
            self.device_picker_loading = true;
            self.device_picker_error = None;
            self.message = Some(("Loading devices...".to_string(), Color::Yellow));
        } else if self.selected_index == 9 {
            self.message = Some(("Starting save sync...".to_string(), Color::Yellow));
        } else if self.selected_index == 7 {
            self.extras_include_manual = !self.extras_include_manual;
            self.message = Some((
                format!(
                    "Extras default (manual): {}",
                    if self.extras_include_manual {
                        "on"
                    } else {
                        "off"
                    }
                ),
                Color::Green,
            ));
        } else if self.selected_index == 6 {
            self.extras_include_cover = !self.extras_include_cover;
            self.message = Some((
                format!(
                    "Extras default (cover): {}",
                    if self.extras_include_cover {
                        "on"
                    } else {
                        "off"
                    }
                ),
                Color::Green,
            ));
        } else if self.selected_index == 5 {
            self.extras_include_related_roms = !self.extras_include_related_roms;
            self.message = Some((
                format!(
                    "Extras default (updates/DLC): {}",
                    if self.extras_include_related_roms {
                        "on"
                    } else {
                        "off"
                    }
                ),
                Color::Green,
            ));
        } else if self.selected_index == 2 {
            // Toggle HTTPS directly and keep the Base URL scheme in sync.
            self.use_https = !self.use_https;
            if self.use_https && self.base_url.starts_with("http://") {
                self.base_url = self.base_url.replace("http://", "https://");
                self.message = Some(("Updated URL scheme (HTTPS)".to_string(), Color::Green));
            } else if !self.use_https && self.base_url.starts_with("https://") {
                self.base_url = self.base_url.replace("https://", "http://");
                self.message = Some(("Updated URL scheme (HTTP)".to_string(), Color::Green));
            }
        } else if self.selected_index == 1 {
            self.path_picker = Some((
                SettingsPickerKind::RomsDir,
                PathPicker::new(PathPickerMode::Directory, self.download_dir.as_str()),
            ));
        } else if self.selected_index == 3 {
            self.path_picker = Some((
                SettingsPickerKind::SaveDir,
                PathPicker::new(PathPickerMode::Directory, self.save_dir.as_str()),
            ));
        } else {
            self.editing = true;
            self.edit_buffer = self.base_url.clone();
            self.edit_cursor = self.edit_buffer.len();
        }
    }

    pub fn save_edit(&mut self) -> bool {
        if !self.editing {
            return true; // UseHttps toggle is "saved" immediately in memory
        }
        if self.selected_index == 0 {
            self.base_url = self.edit_buffer.trim().to_string();
        }
        self.editing = false;
        true
    }

    pub fn cancel_edit(&mut self) {
        self.editing = false;
        self.confirm = None;
        self.path_picker = None;
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

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if let Some((kind, ref mut picker)) = self.path_picker {
            let chunks = Layout::default()
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(area);
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
            let hint =
                "Esc: cancel   Ctrl+Enter: apply typed path (creates folders)   Tab: path/list";
            let title = match kind {
                SettingsPickerKind::RomsDir => "Choose ROMs directory",
                SettingsPickerKind::SaveDir => "Choose save directory",
            };
            picker.render(f, chunks[1], title, hint);
            f.render_widget(
                Paragraph::new("ROMs directory picker — Esc returns without changing")
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL)),
                chunks[2],
            );
            return;
        }

        if self.device_picker_open {
            self.render_device_picker(f, area);
            return;
        }

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
            ListItem::new(format!("Roms Dir:     {}", self.download_dir)),
            ListItem::new(format!(
                "Use HTTPS:    {}",
                if self.use_https { "[X] Yes" } else { "[ ] No" }
            )),
            ListItem::new(format!("Save Dir:     {}", self.save_dir)),
            ListItem::new(format!(
                "Sync Device:  {}",
                self.sync_device_id.as_deref().unwrap_or("(not selected)")
            )),
            ListItem::new("Sync Saves Now"),
            ListItem::new(format!(
                "Extras: incl. updates/DLC (picker default): {}",
                if self.extras_include_related_roms {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            )),
            ListItem::new(format!(
                "Extras: incl. cover (picker default):     {}",
                if self.extras_include_cover {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            )),
            ListItem::new(format!(
                "Extras: incl. manual (picker default):   {}",
                if self.extras_include_manual {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            )),
            ListItem::new(format!(
                "Auth:         {} (Enter to change)",
                self.auth_status
            )),
            ListItem::new("Clear Cache (Remove cached ROM data)"),
            ListItem::new("Reset Configuration (Delete settings from disk & keyring)"),
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
        if let Some(confirm) = &self.confirm {
            let msg = match confirm {
                SettingsConfirm::Reset => {
                    "Are you sure you want to delete all settings? (Enter: Yes, Esc: Cancel)"
                }
                SettingsConfirm::ClearCache => {
                    "Are you sure you want to clear the ROM cache? (Enter: Yes, Esc: Cancel)"
                }
            };
            f.render_widget(
                Paragraph::new(msg)
                    .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                chunks[2],
            );
        } else if let Some((msg, color)) = &self.message {
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
        let help = if self.confirm.is_some() {
            "Enter: confirm   Esc: cancel"
        } else if self.editing {
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
        if let Some((_, ref picker)) = self.path_picker {
            let chunks = Layout::default()
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(area);
            return picker.cursor_position(chunks[1], "Choose ROMs directory");
        }

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
        let label_len = 14; // "Base URL:     ".len()
        let x = list_area.x + 1 /* border */ + 3 /* highlight symbol */ + label_len + self.edit_cursor as u16;

        Some((x, y))
    }

    pub fn set_devices(&mut self, devices: Vec<DeviceSchema>) {
        self.devices = devices;
        self.device_picker_loading = false;
        self.device_picker_error = None;
        self.device_selected_index = self
            .sync_device_id
            .as_ref()
            .and_then(|id| self.devices.iter().position(|d| &d.id == id))
            .unwrap_or(0)
            .min(self.devices.len().saturating_sub(1));
    }

    pub fn set_device_error(&mut self, error: String) {
        self.device_picker_loading = false;
        self.device_picker_error = Some(error);
    }

    pub fn device_next(&mut self) {
        if !self.devices.is_empty() {
            self.device_selected_index =
                (self.device_selected_index + 1).min(self.devices.len() - 1);
        }
    }

    pub fn device_previous(&mut self) {
        self.device_selected_index = self.device_selected_index.saturating_sub(1);
    }

    pub fn confirm_device(&mut self) {
        if let Some(device) = self.devices.get(self.device_selected_index) {
            self.sync_device_id = Some(device.id.clone());
            self.device_picker_open = false;
            self.message = Some((
                "Sync device updated (press S to save)".to_string(),
                Color::Green,
            ));
        }
    }

    fn render_device_picker(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);
        let info = [
            format!(
                "romm-cli: v{} | RomM server: {}",
                self.version, self.server_version
            ),
            "Select the RomM sync device used for manual push-pull.".to_string(),
        ];
        f.render_widget(
            Paragraph::new(info.join("\n")).block(Block::default().borders(Borders::BOTTOM)),
            chunks[0],
        );
        if self.device_picker_loading {
            f.render_widget(
                Paragraph::new("Loading devices...")
                    .block(Block::default().title(" Devices ").borders(Borders::ALL)),
                chunks[1],
            );
        } else if let Some(error) = &self.device_picker_error {
            f.render_widget(
                Paragraph::new(format!("Could not load devices: {error}"))
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().title(" Devices ").borders(Borders::ALL)),
                chunks[1],
            );
        } else {
            let items: Vec<ListItem> = self
                .devices
                .iter()
                .map(|d| {
                    let name = d.name.as_deref().unwrap_or("(unnamed)");
                    ListItem::new(format!("{name}  [{}]  mode={:?}", d.id, d.sync_mode))
                })
                .collect();
            let mut state = ListState::default();
            state.select(Some(self.device_selected_index));
            f.render_stateful_widget(
                List::new(items)
                    .block(Block::default().title(" Devices ").borders(Borders::ALL))
                    .highlight_symbol(">> ")
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                chunks[1],
                &mut state,
            );
        }
        f.render_widget(
            Paragraph::new("Enter: choose   Esc: cancel   ↑/↓: select")
                .block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }
}
