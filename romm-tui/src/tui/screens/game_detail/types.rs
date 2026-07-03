use std::sync::{Arc, Mutex};
use std::time::Instant;

use ratatui_image::picker::ProtocolType;
use ratatui_image::protocol::StatefulProtocol;

use crate::tui::path_picker::PathPicker;
use crate::tui::screens::extras_picker::{ExtrasPickerItem, ExtrasTargetSeed};
use crate::tui::screens::{LibraryBrowseScreen, SearchScreen};
use romm_api::core::download::DownloadJob;
use romm_api::core::extras::collect_update_dlc_files;
use romm_api::types::{AchievementRow, Rom, RomFileCategory, SaveMetadata};

/// Previous screen when opening game detail (so Esc can return).
pub enum GameDetailPrevious {
    Library(Box<LibraryBrowseScreen>),
    Search(SearchScreen),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverRenderMode {
    Auto,
    InlineImage,
    TextFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverState {
    Idle,
    Loading,
    Ready,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveListState {
    Idle,
    Loading,
    Loaded(Vec<SaveMetadata>),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchievementListState {
    Idle,
    Loading,
    Loaded {
        rows: Vec<AchievementRow>,
        summary: (usize, usize),
    },
    Empty(String),
    Failed(String),
    Unsupported(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Info,
    Extras,
    Saves,
    Achievements,
    Technical,
}

impl DetailTab {
    pub const ALL: [DetailTab; 5] = [
        DetailTab::Info,
        DetailTab::Extras,
        DetailTab::Saves,
        DetailTab::Achievements,
        DetailTab::Technical,
    ];

    pub fn index(self) -> usize {
        match self {
            DetailTab::Info => 0,
            DetailTab::Extras => 1,
            DetailTab::Saves => 2,
            DetailTab::Achievements => 3,
            DetailTab::Technical => 4,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            DetailTab::Info => "Info",
            DetailTab::Extras => "Extras",
            DetailTab::Saves => "Saves",
            DetailTab::Achievements => "Achievements",
            DetailTab::Technical => "Technical",
        }
    }
}

pub use romm_api::config::{
    GAME_DETAIL_COVER_PANEL_WIDTH_DEFAULT as COVER_PANEL_WIDTH_DEFAULT,
    GAME_DETAIL_COVER_PANEL_WIDTH_MAX as COVER_PANEL_WIDTH_MAX,
    GAME_DETAIL_COVER_PANEL_WIDTH_MIN as COVER_PANEL_WIDTH_MIN,
};

/// Detailed view for a single ROM (and its related files).
pub struct GameDetailScreen {
    pub rom: Rom,
    /// Other files for the same game (updates, DLC).
    pub other_files: Vec<Rom>,
    pub previous: GameDetailPrevious,
    pub message: Option<String>,
    pub message_clear_at: Option<Instant>,
    /// Shared download list â€” used to show inline progress for this ROM.
    pub downloads: Arc<Mutex<Vec<DownloadJob>>>,
    /// Whether a download has been started from this detail view.
    pub has_started_download: bool,
    /// Whether the user has acknowledged the download completion message.
    pub download_completion_acknowledged: bool,
    pub cover_render_mode: CoverRenderMode,
    pub cover_state: CoverState,
    pub cover_last_url: Option<String>,
    pub cover_protocol: Option<ProtocolType>,
    pub cover_image: Option<StatefulProtocol>,
    pub active_tab: DetailTab,
    pub saves_state: SaveListState,
    pub selected_save_index: usize,
    pub achievements_state: AchievementListState,
    pub selected_achievement_index: usize,
    pub save_upload_picker: Option<PathPicker>,
    pub save_screenshot_state: CoverState,
    pub save_screenshot_image: Option<StatefulProtocol>,
    pub extras_items: Vec<ExtrasPickerItem>,
    pub selected_extras_index: usize,
    /// Pending confirmation before `PUT ?unmatch_metadata=true`.
    pub metadata_unmatch_confirm: bool,
    /// Width of the cover column in terminal cells.
    pub cover_panel_width: u16,
}

impl GameDetailScreen {
    pub fn build_extras_items(rom: &Rom, other_files: &[Rom]) -> Vec<ExtrasPickerItem> {
        let mut items = Vec::new();
        for other in other_files {
            items.push(ExtrasPickerItem {
                label: other.fs_name.clone(),
                sublabel: format!("Related ROM (id {})", other.id),
                checked: true,
                seed: ExtrasTargetSeed::RelatedRom(Box::new(other.clone())),
            });
        }
        for file in collect_update_dlc_files(rom) {
            let tag = match file.category {
                Some(RomFileCategory::Update) => "Update",
                Some(RomFileCategory::Dlc) => "DLC",
                _ => "ROM file",
            };
            items.push(ExtrasPickerItem {
                label: file.file_name.clone(),
                sublabel: format!("{tag} (file id {})", file.id),
                checked: true,
                seed: ExtrasTargetSeed::InternalRomFile(file),
            });
        }
        if rom
            .url_cover
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some()
        {
            items.push(ExtrasPickerItem {
                label: "Cover image".to_string(),
                sublabel: "From url_cover".to_string(),
                checked: true,
                seed: ExtrasTargetSeed::Cover,
            });
        }
        if rom
            .url_manual
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some()
        {
            items.push(ExtrasPickerItem {
                label: "Manual".to_string(),
                sublabel: "From url_manual".to_string(),
                checked: true,
                seed: ExtrasTargetSeed::Manual,
            });
        }
        items
    }
}
