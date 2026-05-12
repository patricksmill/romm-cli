use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Frame;

use crate::config::{disk_has_unresolved_keyring_sentinel, Config};
use crate::endpoints::device::DeviceSchema;
use crate::tui::path_picker::{PathPicker, PathPickerMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Connection,
    Saves,
    Extras,
    AuthMaintenance,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 4] = [
        SettingsTab::Connection,
        SettingsTab::Saves,
        SettingsTab::Extras,
        SettingsTab::AuthMaintenance,
    ];

    pub const COUNT: usize = Self::ALL.len();

    pub fn index(self) -> usize {
        match self {
            SettingsTab::Connection => 0,
            SettingsTab::Saves => 1,
            SettingsTab::Extras => 2,
            SettingsTab::AuthMaintenance => 3,
        }
    }

    fn title(self) -> &'static str {
        match self {
            SettingsTab::Connection => "Connection",
            SettingsTab::Saves => "Saves",
            SettingsTab::Extras => "Extras",
            SettingsTab::AuthMaintenance => "Auth/Maint",
        }
    }

    pub fn rows(self) -> &'static [SettingsRow] {
        match self {
            SettingsTab::Connection => &CONNECTION_ROWS,
            SettingsTab::Saves => &SAVES_ROWS,
            SettingsTab::Extras => &EXTRAS_ROWS,
            SettingsTab::AuthMaintenance => &AUTH_MAINT_ROWS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsRow {
    BaseUrl,
    RomsDir,
    UseHttps,
    SaveDir,
    SyncDevice,
    SyncNow,
    ExtrasRelatedRoms,
    ExtrasCover,
    ExtrasManual,
    Auth,
    ClearCache,
    ResetConfiguration,
}

const CONNECTION_ROWS: [SettingsRow; 3] = [
    SettingsRow::BaseUrl,
    SettingsRow::RomsDir,
    SettingsRow::UseHttps,
];
const SAVES_ROWS: [SettingsRow; 3] = [
    SettingsRow::SaveDir,
    SettingsRow::SyncDevice,
    SettingsRow::SyncNow,
];
const EXTRAS_ROWS: [SettingsRow; 3] = [
    SettingsRow::ExtrasRelatedRoms,
    SettingsRow::ExtrasCover,
    SettingsRow::ExtrasManual,
];
const AUTH_MAINT_ROWS: [SettingsRow; 3] = [
    SettingsRow::Auth,
    SettingsRow::ClearCache,
    SettingsRow::ResetConfiguration,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsPickerKind {
    RomsDir,
    SaveDir,
}

#[derive(Debug, PartialEq, Eq)]
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

    pub selected_tab: SettingsTab,
    selected_indices: [usize; SettingsTab::COUNT],
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
            selected_tab: SettingsTab::Connection,
            selected_indices: [0; SettingsTab::COUNT],
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

    pub fn selected_row_index(&self) -> usize {
        let rows = self.selected_tab.rows();
        self.selected_indices[self.selected_tab.index()].min(rows.len().saturating_sub(1))
    }

    fn set_selected_row_index(&mut self, index: usize) {
        let max = self.selected_tab.rows().len().saturating_sub(1);
        self.selected_indices[self.selected_tab.index()] = index.min(max);
    }

    pub fn selected_row(&self) -> SettingsRow {
        self.selected_tab.rows()[self.selected_row_index()]
    }

    pub fn active_rows(&self) -> &'static [SettingsRow] {
        self.selected_tab.rows()
    }

    pub fn next_tab(&mut self) {
        if self.editing || self.confirm.is_some() {
            return;
        }
        let next = (self.selected_tab.index() + 1) % SettingsTab::COUNT;
        self.selected_tab = SettingsTab::ALL[next];
        self.set_selected_row_index(self.selected_row_index());
    }

    pub fn previous_tab(&mut self) {
        if self.editing || self.confirm.is_some() {
            return;
        }
        let previous = (self.selected_tab.index() + SettingsTab::COUNT - 1) % SettingsTab::COUNT;
        self.selected_tab = SettingsTab::ALL[previous];
        self.set_selected_row_index(self.selected_row_index());
    }

    pub fn next(&mut self) {
        if !self.editing && self.confirm.is_none() {
            let len = self.selected_tab.rows().len();
            if len > 0 {
                self.set_selected_row_index((self.selected_row_index() + 1) % len);
            }
        }
    }

    pub fn previous(&mut self) {
        if !self.editing && self.confirm.is_none() {
            let len = self.selected_tab.rows().len();
            if len == 0 {
                return;
            }
            if self.selected_row_index() == 0 {
                self.set_selected_row_index(len - 1);
            } else {
                self.set_selected_row_index(self.selected_row_index() - 1);
            }
        }
    }

    pub fn enter_edit(&mut self) {
        match self.selected_row() {
            SettingsRow::ResetConfiguration => self.confirm = Some(SettingsConfirm::Reset),
            SettingsRow::ClearCache => self.confirm = Some(SettingsConfirm::ClearCache),
            SettingsRow::SyncDevice => {
                self.device_picker_open = true;
                self.device_picker_loading = true;
                self.device_picker_error = None;
                self.message = Some(("Loading devices...".to_string(), Color::Yellow));
            }
            SettingsRow::SyncNow => {
                self.message = Some(("Starting save sync...".to_string(), Color::Yellow));
            }
            SettingsRow::ExtrasManual => {
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
            }
            SettingsRow::ExtrasCover => {
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
            }
            SettingsRow::ExtrasRelatedRoms => {
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
            }
            SettingsRow::UseHttps => {
                // Toggle HTTPS directly and keep the Base URL scheme in sync.
                self.use_https = !self.use_https;
                if self.use_https && self.base_url.starts_with("http://") {
                    self.base_url = self.base_url.replace("http://", "https://");
                    self.message = Some(("Updated URL scheme (HTTPS)".to_string(), Color::Green));
                } else if !self.use_https && self.base_url.starts_with("https://") {
                    self.base_url = self.base_url.replace("https://", "http://");
                    self.message = Some(("Updated URL scheme (HTTP)".to_string(), Color::Green));
                }
            }
            SettingsRow::RomsDir => {
                self.path_picker = Some((
                    SettingsPickerKind::RomsDir,
                    PathPicker::new(PathPickerMode::Directory, self.download_dir.as_str()),
                ));
            }
            SettingsRow::SaveDir => {
                self.path_picker = Some((
                    SettingsPickerKind::SaveDir,
                    PathPicker::new(PathPickerMode::Directory, self.save_dir.as_str()),
                ));
            }
            SettingsRow::BaseUrl => {
                self.editing = true;
                self.edit_buffer = self.base_url.clone();
                self.edit_cursor = self.edit_buffer.len();
            }
            SettingsRow::Auth => {}
        }
    }

    pub fn save_edit(&mut self) -> bool {
        if !self.editing {
            return true; // UseHttps toggle is "saved" immediately in memory
        }
        if self.selected_row() == SettingsRow::BaseUrl {
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
                Constraint::Length(3), // Settings tabs
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

        // -- Tabs --
        let titles = SettingsTab::ALL
            .iter()
            .map(|tab| Line::from(Span::raw(tab.title())))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.selected_tab.index())
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[1]);

        // -- Editable List --
        let items = self
            .active_rows()
            .iter()
            .copied()
            .map(|row| self.render_row_item(row))
            .collect::<Vec<_>>();

        let mut state = ListState::default();
        state.select(Some(self.selected_row_index()));

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} ", self.selected_tab.title()))
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[2], &mut state);

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
                chunks[3],
            );
        } else if let Some((msg, color)) = &self.message {
            f.render_widget(
                Paragraph::new(msg.as_str()).style(Style::default().fg(*color)),
                chunks[3],
            );
        } else if self.editing {
            f.render_widget(
                Paragraph::new("Editing... Enter: save   Esc: cancel")
                    .style(Style::default().fg(Color::Cyan)),
                chunks[3],
            );
        }

        // -- Footer Help --
        let help = if self.confirm.is_some() {
            "Enter: confirm   Esc: cancel"
        } else if self.editing {
            "Backspace: delete   Arrows: move cursor   Enter: save   Esc: cancel"
        } else {
            "Tab/←/→: tabs   ↑/↓: select   Enter: edit/toggle   S: save to disk   Esc: back"
        };
        f.render_widget(
            Paragraph::new(help).block(Block::default().borders(Borders::ALL)),
            chunks[4],
        );
    }

    fn render_row_item(&self, row: SettingsRow) -> ListItem<'static> {
        match row {
            SettingsRow::BaseUrl => ListItem::new(format!(
                "Base URL:     {}",
                if self.editing && self.selected_row() == SettingsRow::BaseUrl {
                    &self.edit_buffer
                } else {
                    &self.base_url
                }
            )),
            SettingsRow::RomsDir => ListItem::new(format!("Roms Dir:     {}", self.download_dir)),
            SettingsRow::UseHttps => ListItem::new(format!(
                "Use HTTPS:    {}",
                if self.use_https { "[X] Yes" } else { "[ ] No" }
            )),
            SettingsRow::SaveDir => ListItem::new(format!("Save Dir:     {}", self.save_dir)),
            SettingsRow::SyncDevice => ListItem::new(format!(
                "Sync Device:  {}",
                self.sync_device_id.as_deref().unwrap_or("(not selected)")
            )),
            SettingsRow::SyncNow => ListItem::new("Sync Saves Now"),
            SettingsRow::ExtrasRelatedRoms => ListItem::new(format!(
                "Incl. updates/DLC (picker default): {}",
                if self.extras_include_related_roms {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            )),
            SettingsRow::ExtrasCover => ListItem::new(format!(
                "Incl. cover (picker default):       {}",
                if self.extras_include_cover {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            )),
            SettingsRow::ExtrasManual => ListItem::new(format!(
                "Incl. manual (picker default):      {}",
                if self.extras_include_manual {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            )),
            SettingsRow::Auth => ListItem::new(format!(
                "Auth:         {} (Enter to change)",
                self.auth_status
            )),
            SettingsRow::ClearCache => ListItem::new("Clear Cache (Remove cached ROM data)"),
            SettingsRow::ResetConfiguration => {
                ListItem::new("Reset Configuration (Delete settings from disk & keyring)")
            }
        }
    }

    pub fn cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        if let Some((kind, ref picker)) = self.path_picker {
            let chunks = Layout::default()
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(area);
            let title = match kind {
                SettingsPickerKind::RomsDir => "Choose ROMs directory",
                SettingsPickerKind::SaveDir => "Choose save directory",
            };
            return picker.cursor_position(chunks[1], title);
        }

        if !self.editing || self.selected_row() != SettingsRow::BaseUrl {
            return None;
        }

        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let list_area = chunks[2];
        let y = list_area.y + 1 + self.selected_row_index() as u16;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ExtrasDefaults, SaveSyncConfig};

    fn test_config() -> Config {
        Config {
            base_url: "https://romm.example.com".to_string(),
            download_dir: "C:\\roms".to_string(),
            use_https: true,
            auth: None,
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig {
                save_dir: Some("C:\\saves".to_string()),
                device_id: None,
            },
        }
    }

    fn screen() -> SettingsScreen {
        SettingsScreen::new(&test_config(), Some("1.0.0"))
    }

    #[test]
    fn tabs_expose_expected_rows() {
        assert_eq!(
            SettingsTab::Connection.rows(),
            &[
                SettingsRow::BaseUrl,
                SettingsRow::RomsDir,
                SettingsRow::UseHttps
            ]
        );
        assert_eq!(
            SettingsTab::Saves.rows(),
            &[
                SettingsRow::SaveDir,
                SettingsRow::SyncDevice,
                SettingsRow::SyncNow
            ]
        );
        assert_eq!(
            SettingsTab::Extras.rows(),
            &[
                SettingsRow::ExtrasRelatedRoms,
                SettingsRow::ExtrasCover,
                SettingsRow::ExtrasManual
            ]
        );
        assert_eq!(
            SettingsTab::AuthMaintenance.rows(),
            &[
                SettingsRow::Auth,
                SettingsRow::ClearCache,
                SettingsRow::ResetConfiguration
            ]
        );
    }

    #[test]
    fn row_navigation_wraps_within_active_tab() {
        let mut s = screen();

        assert_eq!(s.selected_row(), SettingsRow::BaseUrl);
        s.previous();
        assert_eq!(s.selected_row(), SettingsRow::UseHttps);
        s.next();
        assert_eq!(s.selected_row(), SettingsRow::BaseUrl);
    }

    #[test]
    fn tab_navigation_preserves_per_tab_selection() {
        let mut s = screen();

        s.next();
        s.next();
        assert_eq!(s.selected_row(), SettingsRow::UseHttps);

        s.next_tab();
        assert_eq!(s.selected_tab, SettingsTab::Saves);
        assert_eq!(s.selected_row(), SettingsRow::SaveDir);

        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SyncDevice);

        s.previous_tab();
        assert_eq!(s.selected_tab, SettingsTab::Connection);
        assert_eq!(s.selected_row(), SettingsRow::UseHttps);

        s.next_tab();
        assert_eq!(s.selected_tab, SettingsTab::Saves);
        assert_eq!(s.selected_row(), SettingsRow::SyncDevice);
    }

    #[test]
    fn activation_rows_resolve_to_expected_intents() {
        let mut s = screen();

        s.selected_tab = SettingsTab::AuthMaintenance;
        assert_eq!(s.selected_row(), SettingsRow::Auth);

        s.next();
        assert_eq!(s.selected_row(), SettingsRow::ClearCache);
        s.enter_edit();
        assert_eq!(s.confirm, Some(SettingsConfirm::ClearCache));

        s.cancel_edit();
        s.next();
        assert_eq!(s.selected_row(), SettingsRow::ResetConfiguration);
        s.enter_edit();
        assert_eq!(s.confirm, Some(SettingsConfirm::Reset));
    }

    #[test]
    fn save_action_rows_trigger_matching_state() {
        let mut s = screen();
        s.selected_tab = SettingsTab::Saves;

        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SyncDevice);
        s.enter_edit();
        assert!(s.device_picker_open);
        assert!(s.device_picker_loading);

        s.device_picker_open = false;
        s.device_picker_loading = false;
        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SyncNow);
        s.enter_edit();
        assert_eq!(
            s.message.as_ref().map(|(msg, _)| msg.as_str()),
            Some("Starting save sync...")
        );
    }

    #[test]
    fn extras_rows_toggle_matching_defaults() {
        let mut s = screen();
        s.selected_tab = SettingsTab::Extras;

        s.enter_edit();
        assert!(!s.extras_include_related_roms);
        assert!(s.extras_include_cover);
        assert!(s.extras_include_manual);

        s.next();
        s.enter_edit();
        assert!(!s.extras_include_cover);
        assert!(s.extras_include_manual);

        s.next();
        s.enter_edit();
        assert!(!s.extras_include_manual);
    }
}
