//! Collection of individual TUI screens.
//!
//! Each screen is a small, focused module responsible for rendering a
//! specific view (main menu, library browser, downloads list, etc.) and
//! holding just enough state for that view. The central `App` in
//! `tui::app` chooses which screen is active.

pub mod browse;
pub mod connected_splash;
pub mod download;
pub mod execute;
pub mod game_detail;
pub mod library_browse;
pub mod main_menu;
pub mod result;
pub mod search;
pub mod settings;
pub mod setup_wizard;

pub use browse::BrowseScreen;
pub use download::DownloadScreen;
pub use execute::ExecuteScreen;
pub use game_detail::{GameDetailPrevious, GameDetailScreen};
pub use library_browse::LibraryBrowseScreen;
pub use main_menu::MainMenuScreen;
pub use result::{ResultDetailScreen, ResultScreen};
pub use search::SearchScreen;
pub use settings::SettingsScreen;
