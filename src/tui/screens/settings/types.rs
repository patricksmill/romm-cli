use std::collections::HashMap;

use ratatui::style::Color;

use crate::endpoints::device::DeviceSchema;
use crate::feature_compat::SaveSyncCompatibility;
use crate::tui::path_picker::PathPicker;
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

    pub(crate) fn title(self) -> &'static str {
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

pub(crate) const CONNECTION_ROWS: [SettingsRow; 2] = [SettingsRow::BaseUrl, SettingsRow::UseHttps];
pub(crate) const SAVES_ROWS: [SettingsRow; 4] = [
    SettingsRow::SaveDir,
    SettingsRow::SaveConsolePaths,
    SettingsRow::SyncDevice,
    SettingsRow::SyncNow,
];
pub(crate) const EXTRAS_ROWS: [SettingsRow; 3] = [
    SettingsRow::ExtrasRelatedRoms,
    SettingsRow::ExtrasCover,
    SettingsRow::ExtrasManual,
];
pub(crate) const AUTH_MAINT_ROWS: [SettingsRow; 3] = [
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
    pub(crate) selected_indices: [usize; SettingsTab::COUNT],
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
