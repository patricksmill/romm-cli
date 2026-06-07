use crate::core::utils::RomGroup;
use crate::tui::path_picker::{PathPicker, PathPickerMode};
use crate::types::{Collection, Platform, RomList};

/// File path picker for TUI upload (single ROM file).
#[derive(Debug)]
pub struct UploadPrompt {
    pub picker: PathPicker,
    /// When true, run `scan_library` with wait after upload (matches CLI `roms upload --scan --wait`).
    pub scan_after: bool,
}

impl Default for UploadPrompt {
    fn default() -> Self {
        Self {
            picker: PathPicker::new(PathPickerMode::File, ""),
            scan_after: true,
        }
    }
}

/// Which high-level grouping is currently shown in the left pane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibrarySubsection {
    ByConsole,
    ByCollection,
}

/// Which side of the library view currently has focus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibraryViewMode {
    /// Left panel: list of consoles or collections
    List,
    /// Right panel: list of ROMs for selected console/collection
    Roms,
}

pub use crate::config::{
    LIBRARY_LEFT_PANEL_PERCENT_DEFAULT as LEFT_PANEL_PERCENT_DEFAULT,
    LIBRARY_LEFT_PANEL_PERCENT_MAX as LEFT_PANEL_PERCENT_MAX,
    LIBRARY_LEFT_PANEL_PERCENT_MIN as LEFT_PANEL_PERCENT_MIN,
};

/// Main library browser: consoles/collections on the left, games on the right.
pub struct LibraryBrowseScreen {
    pub platforms: Vec<Platform>,
    pub collections: Vec<Collection>,
    pub subsection: LibrarySubsection,
    pub list_index: usize,
    pub view_mode: LibraryViewMode,
    pub roms: Option<RomList>,
    /// One row per game name (base + updates/DLC grouped).
    pub rom_groups: Option<Vec<RomGroup>>,
    pub rom_selected: usize,
    pub scroll_offset: usize,
    /// Visible data rows in the ROM pane (updated at render time).
    pub(crate) visible_rows: usize,
    /// Non-blocking status from metadata refresh (API warnings, "updated", etc.).
    pub metadata_footer: Option<String>,
    /// When the footer should be automatically cleared.
    pub metadata_footer_clear_at: Option<std::time::Instant>,
    /// True only while ROM data for the current selection is actively loading.
    pub rom_loading: bool,
    /// Modal path entry for uploading a ROM to the selected console (`None` when closed).
    pub upload_prompt: Option<UploadPrompt>,
    /// Horizontal split: left pane width as a percentage of the library area.
    pub left_panel_percent: u16,
}
