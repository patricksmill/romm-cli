//! Per-screen key → action mapping.

use crossterm::event::KeyEvent;

use super::super::event::Action;
use super::super::App;

pub(in crate::tui::app) fn map_screen_key(app: &App, key: &KeyEvent) -> Vec<Action> {
    use super::super::AppScreen;
    match &app.screen {
        AppScreen::MainMenu(_) => super::menu::map_main_menu_key(key),
        AppScreen::LibraryBrowse(_) => vec![Action::LibraryKey(*key)],
        AppScreen::Search(_) => vec![Action::SearchKey(*key)],
        AppScreen::Settings(_) => vec![Action::SettingsKey(*key)],
        AppScreen::GameDetail(_) => vec![Action::GameDetailKey(*key)],
        AppScreen::ExtrasPicker(_) => vec![Action::ExtrasPickerKey(*key)],
        AppScreen::Download(_) => super::download::map_download_key(key),
        AppScreen::SetupWizard(_) => vec![Action::SetupWizardKey(*key)],
    }
}
