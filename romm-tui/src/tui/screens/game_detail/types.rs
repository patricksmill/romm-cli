use std::sync::{Arc, Mutex};
use std::time::Instant;

use ratatui_image::picker::ProtocolType;
use ratatui_image::protocol::StatefulProtocol;

use crate::tui::path_picker::PathPicker;
use crate::tui::screens::{LibraryBrowseScreen, SearchScreen};
use romm_api::core::download::DownloadJob;
use romm_api::types::{Rom, SaveMetadata};

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
    pub show_technical: bool,
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
    pub saves_state: SaveListState,
    pub selected_save_index: usize,
    pub save_upload_picker: Option<PathPicker>,
    /// Pending confirmation before `PUT ?unmatch_metadata=true`.
    pub metadata_unmatch_confirm: bool,
    /// Width of the cover column in terminal cells.
    pub cover_panel_width: u16,
}
