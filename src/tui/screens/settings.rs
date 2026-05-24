use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Frame;

use crate::config::{
    disk_has_unresolved_keyring_sentinel, Config, RomsLayoutConfig, SaveSyncConfig,
};
use crate::core::utils;
use crate::endpoints::device::DeviceSchema;
use crate::feature_compat::SaveSyncCompatibility;
use crate::tui::path_picker::{PathPicker, PathPickerMode};
use crate::types::Platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Connection,
    Roms,
    Saves,
    Extras,
    AuthMaintenance,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 5] = [
        SettingsTab::Connection,
        SettingsTab::Roms,
        SettingsTab::Saves,
        SettingsTab::Extras,
        SettingsTab::AuthMaintenance,
    ];

    pub const COUNT: usize = Self::ALL.len();

    pub fn index(self) -> usize {
        match self {
            SettingsTab::Connection => 0,
            SettingsTab::Roms => 1,
            SettingsTab::Saves => 2,
            SettingsTab::Extras => 3,
            SettingsTab::AuthMaintenance => 4,
        }
    }

    fn title(self) -> &'static str {
        match self {
            SettingsTab::Connection => "Connection",
            SettingsTab::Roms => "ROMs",
            SettingsTab::Saves => "Saves",
            SettingsTab::Extras => "Extras",
            SettingsTab::AuthMaintenance => "Auth/Maint",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsRow {
    BaseUrl,
    RomsDir,
    ConsolePaths,
    UseHttps,
    SaveDir,
    SaveConsolePaths,
    SyncDevice,
    SyncNow,
    ExtrasRelatedRoms,
    ExtrasCover,
    ExtrasManual,
    Auth,
    ClearCache,
    ResetConfiguration,
}

const CONNECTION_ROWS: [SettingsRow; 2] = [SettingsRow::BaseUrl, SettingsRow::UseHttps];
const SAVES_ROWS: [SettingsRow; 4] = [
    SettingsRow::SaveDir,
    SettingsRow::SaveConsolePaths,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsolePathKind {
    Roms,
    Saves,
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
    pub save_sync_compat: SaveSyncCompatibility,
    pub rom_platform_dirs: HashMap<u64, String>,
    pub save_platform_dirs: HashMap<u64, String>,
    pub console_picker_open: bool,
    pub active_console_kind: Option<ConsolePathKind>,
    pub console_picker_loading: bool,
    pub console_picker_error: Option<String>,
    pub console_platforms: Vec<Platform>,
    pub console_selected_index: usize,
    /// Per-console directory browser (`None` when not picking for a platform).
    pub console_path_picker: Option<(u64, PathPicker)>,
}

impl SettingsScreen {
    pub fn new(
        config: &Config,
        romm_server_version: Option<&str>,
        save_sync_compat: SaveSyncCompatibility,
    ) -> Self {
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
            save_sync_compat,
            rom_platform_dirs: config.roms_layout.platform_dirs.clone(),
            save_platform_dirs: config.save_sync.platform_dirs.clone(),
            console_picker_open: false,
            active_console_kind: None,
            console_picker_loading: false,
            console_picker_error: None,
            console_platforms: Vec::new(),
            console_selected_index: 0,
            console_path_picker: None,
        }
    }

    pub fn roms_layout_config(&self) -> RomsLayoutConfig {
        let mut layout = RomsLayoutConfig::default();
        layout.platform_dirs = self.rom_platform_dirs.clone();
        layout
    }

    pub fn save_sync_config(&self) -> SaveSyncConfig {
        SaveSyncConfig {
            save_dir: Some(self.save_dir.clone()),
            device_id: self.sync_device_id.clone(),
            platform_dirs: self.save_platform_dirs.clone(),
        }
    }

    fn console_dirs(&self, kind: ConsolePathKind) -> &HashMap<u64, String> {
        match kind {
            ConsolePathKind::Roms => &self.rom_platform_dirs,
            ConsolePathKind::Saves => &self.save_platform_dirs,
        }
    }

    fn console_dirs_mut(&mut self, kind: ConsolePathKind) -> &mut HashMap<u64, String> {
        match kind {
            ConsolePathKind::Roms => &mut self.rom_platform_dirs,
            ConsolePathKind::Saves => &mut self.save_platform_dirs,
        }
    }

    pub fn visible_rows(&self) -> Vec<SettingsRow> {
        match self.selected_tab {
            SettingsTab::Connection => CONNECTION_ROWS.to_vec(),
            SettingsTab::Roms => vec![SettingsRow::RomsDir, SettingsRow::ConsolePaths],
            SettingsTab::Saves => SAVES_ROWS.to_vec(),
            SettingsTab::Extras => EXTRAS_ROWS.to_vec(),
            SettingsTab::AuthMaintenance => AUTH_MAINT_ROWS.to_vec(),
        }
    }

    fn platform_display_name(platform: &Platform) -> String {
        platform
            .custom_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(platform.name.as_str())
            .to_string()
    }

    fn auto_console_dir_preview(&self, kind: ConsolePathKind, platform: &Platform) -> String {
        let slug = platform.fs_slug.as_str();
        let base = match kind {
            ConsolePathKind::Roms => self.download_dir.trim_end_matches(['/', '\\']),
            ConsolePathKind::Saves => self.save_dir.trim_end_matches(['/', '\\']),
        };
        format!("{}/{}", base, utils::sanitize_filename(slug))
    }

    fn console_dir_preview(&self, kind: ConsolePathKind, platform: &Platform) -> String {
        self.console_dirs(kind)
            .get(&platform.id)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| self.auto_console_dir_preview(kind, platform))
    }

    pub fn open_console_picker(&mut self, kind: ConsolePathKind) {
        self.console_selected_index = 0;
        self.console_picker_open = true;
        self.active_console_kind = Some(kind);
        self.console_path_picker = None;
        self.console_picker_loading = true;
        self.console_picker_error = None;
        self.console_platforms.clear();
    }

    pub fn set_console_platforms(&mut self, platforms: Vec<Platform>) {
        self.console_platforms = platforms;
        self.console_picker_loading = false;
        self.console_picker_error = None;
        self.console_selected_index = self
            .console_selected_index
            .min(self.console_platforms.len().saturating_sub(1));
    }

    pub fn set_console_platform_error(&mut self, error: String) {
        self.console_picker_loading = false;
        self.console_picker_error = Some(error);
    }

    pub fn clear_console_path(&mut self, platform_id: u64) {
        let Some(kind) = self.active_console_kind else {
            return;
        };
        self.console_dirs_mut(kind).remove(&platform_id);
        self.message = Some((
            "Custom path cleared (press S to save)".to_string(),
            Color::Green,
        ));
    }

    pub fn console_next(&mut self) {
        if !self.console_platforms.is_empty() {
            self.console_selected_index =
                (self.console_selected_index + 1).min(self.console_platforms.len() - 1);
        }
    }

    pub fn console_previous(&mut self) {
        self.console_selected_index = self.console_selected_index.saturating_sub(1);
    }

    pub fn open_console_path_picker(&mut self) {
        let Some(kind) = self.active_console_kind else {
            return;
        };
        let Some(platform) = self.console_platforms.get(self.console_selected_index) else {
            return;
        };
        let initial = self.console_dir_preview(kind, platform);
        self.console_path_picker = Some((
            platform.id,
            PathPicker::new(PathPickerMode::Directory, &initial),
        ));
    }

    pub fn confirm_console_path(&mut self, platform_id: u64, path: String) {
        let Some(kind) = self.active_console_kind else {
            return;
        };
        self.console_dirs_mut(kind).insert(platform_id, path);
        self.console_path_picker = None;
        let label = match kind {
            ConsolePathKind::Roms => "Custom console path updated (press S to save)",
            ConsolePathKind::Saves => "Custom save path updated (press S to save)",
        };
        self.message = Some((label.to_string(), Color::Green));
    }

    pub fn save_sync_supported(&self) -> bool {
        self.save_sync_compat.supported
    }

    pub fn set_save_sync_unsupported_message(&mut self) {
        self.message = Some((self.save_sync_compat.unsupported_message(), Color::Yellow));
    }

    pub fn selected_row_index(&self) -> usize {
        let rows = self.visible_rows();
        self.selected_indices[self.selected_tab.index()].min(rows.len().saturating_sub(1))
    }

    fn set_selected_row_index(&mut self, index: usize) {
        let max = self.visible_rows().len().saturating_sub(1);
        self.selected_indices[self.selected_tab.index()] = index.min(max);
    }

    pub fn selected_row(&self) -> SettingsRow {
        let rows = self.visible_rows();
        rows[self.selected_row_index()]
    }

    pub fn active_rows(&self) -> &[SettingsRow] {
        // Legacy helper for tests; prefer visible_rows().
        match self.selected_tab {
            SettingsTab::Connection => &CONNECTION_ROWS,
            SettingsTab::Saves => &SAVES_ROWS,
            SettingsTab::Extras => &EXTRAS_ROWS,
            SettingsTab::AuthMaintenance => &AUTH_MAINT_ROWS,
            SettingsTab::Roms => &[],
        }
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
            let len = self.visible_rows().len();
            if len > 0 {
                self.set_selected_row_index((self.selected_row_index() + 1) % len);
            }
        }
    }

    pub fn previous(&mut self) {
        if !self.editing && self.confirm.is_none() {
            let len = self.visible_rows().len();
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
                if !self.save_sync_supported() {
                    self.set_save_sync_unsupported_message();
                    return;
                }
                self.device_picker_open = true;
                self.device_picker_loading = true;
                self.device_picker_error = None;
                self.message = Some(("Loading devices...".to_string(), Color::Yellow));
            }
            SettingsRow::SyncNow => {
                if !self.save_sync_supported() {
                    self.set_save_sync_unsupported_message();
                    return;
                }
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
            SettingsRow::ConsolePaths | SettingsRow::SaveConsolePaths => {}
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
        self.console_path_picker = None;
        self.console_picker_open = false;
        self.active_console_kind = None;
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
        if let Some((platform_id, ref mut picker)) = self.console_path_picker {
            let chunks = Layout::default()
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(area);
            let platform_name = self
                .console_platforms
                .iter()
                .find(|p| p.id == platform_id)
                .map(Self::platform_display_name)
                .unwrap_or_else(|| format!("Platform {platform_id}"));
            let info = [
                format!(
                    "romm-cli: v{} | RomM server: {}",
                    self.version, self.server_version
                ),
                format!("Console: {platform_name}"),
            ];
            f.render_widget(
                Paragraph::new(info.join("\n")).block(Block::default().borders(Borders::BOTTOM)),
                chunks[0],
            );
            picker.render(
                f,
                chunks[1],
                "Choose console directory",
                "Esc: cancel   Ctrl+Enter: apply typed path (creates folders)   Tab: path/list",
            );
            f.render_widget(
                Paragraph::new("Console directory picker — Esc returns without changing")
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL)),
                chunks[2],
            );
            return;
        }

        if self.console_picker_open {
            self.render_console_picker(f, area);
            return;
        }

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
            .visible_rows()
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
        let label = self.row_label(row);
        match row {
            SettingsRow::SyncDevice | SettingsRow::SyncNow if !self.save_sync_supported() => {
                ListItem::new(label).style(Style::default().fg(Color::DarkGray))
            }
            _ => ListItem::new(label),
        }
    }

    fn row_label(&self, row: SettingsRow) -> String {
        let label = match row {
            SettingsRow::BaseUrl => format!(
                "Base URL:     {}",
                if self.editing && self.selected_row() == SettingsRow::BaseUrl {
                    &self.edit_buffer
                } else {
                    &self.base_url
                }
            ),
            SettingsRow::RomsDir => format!("Roms Dir:     {}", self.download_dir),
            SettingsRow::ConsolePaths => {
                let mapped = self.rom_platform_dirs.len();
                format!("Console paths: {mapped} custom · Enter to edit")
            }
            SettingsRow::SaveConsolePaths => {
                let mapped = self.save_platform_dirs.len();
                format!("Save console paths: {mapped} custom · Enter to edit")
            }
            SettingsRow::UseHttps => format!(
                "Use HTTPS:    {}",
                if self.use_https { "[X] Yes" } else { "[ ] No" }
            ),
            SettingsRow::SaveDir => format!("Save Dir:     {}", self.save_dir),
            SettingsRow::SyncDevice => format!(
                "Sync Device:  {}",
                self.sync_device_id.as_deref().unwrap_or("(not selected)")
            ),
            SettingsRow::SyncNow => "Sync Saves Now".to_string(),
            SettingsRow::ExtrasRelatedRoms => format!(
                "Incl. updates/DLC (picker default): {}",
                if self.extras_include_related_roms {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            ),
            SettingsRow::ExtrasCover => format!(
                "Incl. cover (picker default):       {}",
                if self.extras_include_cover {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            ),
            SettingsRow::ExtrasManual => format!(
                "Incl. manual (picker default):      {}",
                if self.extras_include_manual {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            ),
            SettingsRow::Auth => format!("Auth:         {} (Enter to change)", self.auth_status),
            SettingsRow::ClearCache => "Clear Cache (Remove cached ROM data)".to_string(),
            SettingsRow::ResetConfiguration => {
                "Reset Configuration (Delete settings from disk & keyring)".to_string()
            }
        };
        if matches!(row, SettingsRow::SyncDevice | SettingsRow::SyncNow)
            && !self.save_sync_supported()
        {
            format!("{label} (requires newer RomM server)")
        } else {
            label
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

    fn render_console_picker(&mut self, f: &mut Frame, area: Rect) {
        let kind = self.active_console_kind.unwrap_or(ConsolePathKind::Roms);
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);
        let subtitle = match kind {
            ConsolePathKind::Roms => "Set a custom ROM path for consoles on other drives.",
            ConsolePathKind::Saves => "Set a custom save path for consoles on other drives.",
        };
        let info = [
            format!(
                "romm-cli: v{} | RomM server: {}",
                self.version, self.server_version
            ),
            subtitle.to_string(),
        ];
        f.render_widget(
            Paragraph::new(info.join("\n")).block(Block::default().borders(Borders::BOTTOM)),
            chunks[0],
        );
        if self.console_picker_loading {
            f.render_widget(
                Paragraph::new("Loading platforms...")
                    .block(Block::default().title(" Consoles ").borders(Borders::ALL)),
                chunks[1],
            );
        } else if let Some(error) = &self.console_picker_error {
            f.render_widget(
                Paragraph::new(format!("Could not load platforms: {error}"))
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().title(" Consoles ").borders(Borders::ALL)),
                chunks[1],
            );
        } else if self.console_platforms.is_empty() {
            f.render_widget(
                Paragraph::new("No platforms returned from the server.")
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().title(" Consoles ").borders(Borders::ALL)),
                chunks[1],
            );
        } else {
            let items: Vec<ListItem> = self
                .console_platforms
                .iter()
                .map(|platform| {
                    let name = Self::platform_display_name(platform);
                    let path = self.console_dir_preview(kind, platform);
                    let custom = self
                        .console_dirs(kind)
                        .get(&platform.id)
                        .is_some_and(|s| !s.trim().is_empty());
                    let tag = if custom {
                        "custom path"
                    } else {
                        "base default"
                    };
                    ListItem::new(format!("{name}  [{tag}]  {path}"))
                })
                .collect();
            let mut state = ListState::default();
            state.select(Some(self.console_selected_index));
            f.render_stateful_widget(
                List::new(items)
                    .block(Block::default().title(" Consoles ").borders(Borders::ALL))
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
            Paragraph::new("Enter: set path   Del: clear custom   Esc: back   ↑/↓: select")
                .block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::config::{ExtrasDefaults, SaveSyncConfig};
    use crate::feature_compat::{
        supported_save_sync_compatibility, FeatureCompatibility, RequiredEndpoint,
        SAVE_SYNC_FEATURE, SAVE_SYNC_UNSUPPORTED_MESSAGE,
    };

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
                platform_dirs: HashMap::new(),
            },
            roms_layout: RomsLayoutConfig::default(),
        }
    }

    fn screen() -> SettingsScreen {
        SettingsScreen::new(
            &test_config(),
            Some("1.0.0"),
            supported_save_sync_compatibility(),
        )
    }

    fn unsupported_screen() -> SettingsScreen {
        SettingsScreen::new(
            &test_config(),
            Some("1.0.0"),
            FeatureCompatibility::from_registry(
                SAVE_SYNC_FEATURE,
                SAVE_SYNC_UNSUPPORTED_MESSAGE,
                &[RequiredEndpoint {
                    method: "GET",
                    path: "/api/devices",
                }],
                &crate::openapi::EndpointRegistry::default(),
            ),
        )
    }

    #[test]
    fn tabs_expose_expected_rows() {
        let s = screen();
        assert_eq!(
            s.visible_rows(),
            vec![SettingsRow::BaseUrl, SettingsRow::UseHttps]
        );
        assert_eq!(
            CONNECTION_ROWS,
            [SettingsRow::BaseUrl, SettingsRow::UseHttps]
        );
        assert_eq!(SettingsTab::Saves as usize, SettingsTab::Roms.index() + 1);
        assert_eq!(
            SAVES_ROWS,
            [
                SettingsRow::SaveDir,
                SettingsRow::SaveConsolePaths,
                SettingsRow::SyncDevice,
                SettingsRow::SyncNow
            ]
        );
        assert_eq!(
            EXTRAS_ROWS,
            [
                SettingsRow::ExtrasRelatedRoms,
                SettingsRow::ExtrasCover,
                SettingsRow::ExtrasManual
            ]
        );
        assert_eq!(
            AUTH_MAINT_ROWS,
            [
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
    fn saves_tab_shows_save_console_paths_row() {
        let mut s = screen();
        s.selected_tab = SettingsTab::Saves;
        assert_eq!(
            s.visible_rows(),
            vec![
                SettingsRow::SaveDir,
                SettingsRow::SaveConsolePaths,
                SettingsRow::SyncDevice,
                SettingsRow::SyncNow,
            ]
        );
        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SaveConsolePaths);
    }

    #[test]
    fn roms_tab_always_shows_console_paths_row() {
        let mut s = screen();
        s.selected_tab = SettingsTab::Roms;
        assert_eq!(
            s.visible_rows(),
            vec![SettingsRow::RomsDir, SettingsRow::ConsolePaths]
        );
        s.next();
        assert_eq!(s.selected_row(), SettingsRow::ConsolePaths);
    }

    #[test]
    fn tab_navigation_preserves_per_tab_selection() {
        let mut s = screen();

        s.next();
        assert_eq!(s.selected_row(), SettingsRow::UseHttps);

        s.next_tab();
        assert_eq!(s.selected_tab, SettingsTab::Roms);
        assert_eq!(s.selected_row(), SettingsRow::RomsDir);

        s.next_tab();
        assert_eq!(s.selected_tab, SettingsTab::Saves);
        assert_eq!(s.selected_row(), SettingsRow::SaveDir);

        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SaveConsolePaths);

        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SyncDevice);

        s.previous_tab();
        assert_eq!(s.selected_tab, SettingsTab::Roms);
        assert_eq!(s.selected_row(), SettingsRow::RomsDir);

        s.previous_tab();
        assert_eq!(s.selected_tab, SettingsTab::Connection);
        assert_eq!(s.selected_row(), SettingsRow::UseHttps);

        s.next_tab();
        assert_eq!(s.selected_tab, SettingsTab::Roms);
        assert_eq!(s.selected_row(), SettingsRow::RomsDir);

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

    #[test]
    fn unsupported_save_sync_rows_do_not_open_device_picker_or_start_sync() {
        let mut s = unsupported_screen();
        s.selected_tab = SettingsTab::Saves;

        s.next();
        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SyncDevice);
        s.enter_edit();
        assert!(!s.device_picker_open);
        assert_eq!(
            s.message.as_ref().map(|(msg, _)| msg.as_str()),
            Some(
                "This RomM server does not expose save-sync endpoints; upgrade RomM to use romm-cli sync. Missing endpoint(s): GET /api/devices"
            )
        );

        s.next();
        assert_eq!(s.selected_row(), SettingsRow::SyncNow);
        s.enter_edit();
        assert!(!s.sync_inflight);
        assert!(s
            .message
            .as_ref()
            .map(|(msg, _)| msg.contains(SAVE_SYNC_UNSUPPORTED_MESSAGE))
            .unwrap_or(false));
    }

    #[test]
    fn unsupported_save_sync_rows_render_requires_newer_server_annotation() {
        let s = unsupported_screen();

        assert!(s
            .row_label(SettingsRow::SyncDevice)
            .contains("requires newer RomM server"));
        assert!(s
            .row_label(SettingsRow::SyncNow)
            .contains("requires newer RomM server"));
    }
}
