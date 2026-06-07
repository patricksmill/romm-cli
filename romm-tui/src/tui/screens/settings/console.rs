use crate::tui::path_picker::{PathPicker, PathPickerMode};
use romm_api::core::utils;
use romm_api::types::Platform;

use romm_api::endpoints::device::DeviceSchema;

use super::types::{ConsolePathKind, SettingsScreen};
use crate::tui::theme::MessageTone;

impl SettingsScreen {
    pub(crate) fn platform_display_name(platform: &Platform) -> String {
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

    pub(crate) fn console_dir_preview(&self, kind: ConsolePathKind, platform: &Platform) -> String {
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
            MessageTone::Success,
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
        self.message = Some((label.to_string(), MessageTone::Success));
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
                MessageTone::Success,
            ));
        }
    }
}
