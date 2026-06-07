use std::collections::HashMap;

use crate::config::{
    disk_has_unresolved_keyring_sentinel, Config, RomsLayoutConfig, SaveSyncConfig,
};
use crate::feature_compat::SaveSyncCompatibility;
use crate::tui::path_picker::{PathPicker, PathPickerMode};

use super::types::{
    ConsolePathKind, SettingsConfirm, SettingsPickerKind, SettingsRow, SettingsScreen, SettingsTab,
    APPEARANCE_ROWS, AUTH_MAINT_ROWS, CONNECTION_ROWS, EXTRAS_ROWS, SAVES_ROWS,
};
use crate::tui::theme::{next_theme_id, prev_theme_id, theme_display_name, MessageTone};

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
            theme_id: config.theme.clone(),
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

    /// Whether the on-screen values differ from the last saved in-memory config.
    pub fn has_unsaved_changes(&self, saved: &Config) -> bool {
        if self.base_url != saved.base_url {
            return true;
        }
        if self.download_dir != saved.download_dir {
            return true;
        }
        if self.use_https != saved.use_https {
            return true;
        }
        if self.extras_include_related_roms != saved.extras_defaults.include_related_roms {
            return true;
        }
        if self.extras_include_cover != saved.extras_defaults.include_cover {
            return true;
        }
        if self.extras_include_manual != saved.extras_defaults.include_manual {
            return true;
        }
        if self.theme_id != saved.theme {
            return true;
        }
        if self.roms_layout_config() != saved.roms_layout {
            return true;
        }
        if self.sync_device_id != saved.save_sync.device_id {
            return true;
        }
        if self.save_platform_dirs != saved.save_sync.platform_dirs {
            return true;
        }
        let saved_save_dir = crate::config::resolved_save_dir(saved)
            .display()
            .to_string();
        self.save_dir != saved_save_dir
    }

    pub(crate) fn console_dirs(&self, kind: ConsolePathKind) -> &HashMap<u64, String> {
        match kind {
            ConsolePathKind::Roms => &self.rom_platform_dirs,
            ConsolePathKind::Saves => &self.save_platform_dirs,
        }
    }

    pub(crate) fn console_dirs_mut(&mut self, kind: ConsolePathKind) -> &mut HashMap<u64, String> {
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
            SettingsTab::Appearance => APPEARANCE_ROWS.to_vec(),
            SettingsTab::AuthMaintenance => AUTH_MAINT_ROWS.to_vec(),
        }
    }

    pub fn save_sync_supported(&self) -> bool {
        self.save_sync_compat.supported
    }

    pub fn set_save_sync_unsupported_message(&mut self) {
        self.message = Some((
            self.save_sync_compat.unsupported_message(),
            MessageTone::Warning,
        ));
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
            SettingsTab::Appearance => &APPEARANCE_ROWS,
            SettingsTab::AuthMaintenance => &AUTH_MAINT_ROWS,
            SettingsTab::Roms => &[],
        }
    }

    pub fn cycle_theme_next(&mut self) {
        self.theme_id = next_theme_id(&self.theme_id);
    }

    pub fn cycle_theme_prev(&mut self) {
        self.theme_id = prev_theme_id(&self.theme_id);
    }

    pub fn theme_display_name(&self) -> String {
        theme_display_name(&self.theme_id)
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
                self.message = Some(("Loading devices...".to_string(), MessageTone::Warning));
            }
            SettingsRow::SyncNow => {
                if !self.save_sync_supported() {
                    self.set_save_sync_unsupported_message();
                    return;
                }
                self.message = Some(("Starting save sync...".to_string(), MessageTone::Warning));
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
                    MessageTone::Success,
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
                    MessageTone::Success,
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
                    MessageTone::Success,
                ));
            }
            SettingsRow::UseHttps => {
                // Toggle HTTPS directly and keep the Base URL scheme in sync.
                self.use_https = !self.use_https;
                if self.use_https && self.base_url.starts_with("http://") {
                    self.base_url = self.base_url.replace("http://", "https://");
                    self.message = Some((
                        "Updated URL scheme (HTTPS)".to_string(),
                        MessageTone::Success,
                    ));
                } else if !self.use_https && self.base_url.starts_with("https://") {
                    self.base_url = self.base_url.replace("https://", "http://");
                    self.message = Some((
                        "Updated URL scheme (HTTP)".to_string(),
                        MessageTone::Success,
                    ));
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
            SettingsRow::Theme => {}
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
}
