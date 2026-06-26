//! Collection of individual TUI screens.
//!
//! Each screen is a small, focused module responsible for rendering a
//! specific view (main menu, library browser, downloads list, etc.) and
//! holding just enough state for that view. The central `App` in
//! `tui::app` chooses which screen is active.

pub mod connected_splash;
pub mod download;
pub mod extras_picker;
pub mod game_detail;
pub mod library_browse;
pub mod metadata_match;
pub mod search;
pub mod settings;
pub mod setup_wizard;

pub use download::DownloadScreen;
pub use extras_picker::ExtrasPickerScreen;
pub use game_detail::{GameDetailPrevious, GameDetailScreen};
pub use library_browse::LibraryBrowseScreen;
pub use metadata_match::MetadataMatchScreen;
pub use search::SearchScreen;
pub use settings::SettingsScreen;
