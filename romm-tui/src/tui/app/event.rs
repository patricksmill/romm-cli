//! TUI event and action types (Gap 5: Event → Action → update pipeline).

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use romm_api::core::library_scan::ScanCacheInvalidate;
use romm_api::error::RommError;

use super::background::types::{
    AchievementLoadDone, CollectionPrefetchDone, CoverLoadDone, DeviceListDone,
    LibraryMetadataRefreshDone, LibraryUploadComplete, MetadataApplyDone, MetadataSearchDone,
    PlatformListDone, RomLoadDone, SaveDownloadDone, SaveListDone, SaveUploadDone, SearchLoadDone,
    SyncPushPullDone,
};
use crate::tui::keyboard_help::{map_keyboard_help_key, KeyboardHelpInput};

use super::App;

/// Raw or derived input events for the main TUI loop.
#[derive(Debug)]
pub(crate) enum AppEvent {
    Key(KeyEvent),
    #[allow(dead_code)]
    Tick,
    #[allow(dead_code)]
    Paste(String),
    Background(BackgroundAction),
    AutoDismissSplash,
}

/// Background worker completions and scheduler ticks drained each frame.
#[derive(Debug)]
pub(crate) enum BackgroundAction {
    LibraryMetadataRefresh(LibraryMetadataRefreshDone),
    RomLoad(RomLoadDone),
    CollectionPrefetch(CollectionPrefetchDone),
    SearchLoad(SearchLoadDone),
    CoverLoad(CoverLoadDone),
    SaveList(SaveListDone),
    AchievementLoad(AchievementLoadDone),
    SaveUpload(SaveUploadDone),
    SaveDownload(SaveDownloadDone),
    MetadataSearch(MetadataSearchDone),
    MetadataApply(MetadataApplyDone),
    DeviceList(DeviceListDone),
    PlatformList(PlatformListDone),
    SyncPushPull(SyncPushPullDone),
    LibraryUploadProgress { uploaded: u64, total: u64 },
    LibraryUploadDone(Result<LibraryUploadComplete, RommError>),
    LibraryScanDone(Result<(), RommError>),
    DrivePrefetch,
    PollFooterClear,
}

/// Semantic intents applied by [`super::App::update`].
#[derive(Debug)]
pub(crate) enum Action {
    Quit,
    DismissGlobalMessage,
    DismissStartupSplash,
    ShowKeyboardHelp,
    HideKeyboardHelp,
    KeyboardHelpInput(KeyboardHelpInput),
    ToggleDownloadOverlay,
    CloseDownloadOverlay,
    ToggleSearchOverlay,
    ToggleSettingsOverlay,
    RescanLibrary(ScanCacheInvalidate),
    ToggleLibraryUploadPrompt,
    ProcessDeferredRomLoad,
    ApplyStartupUpdate,
    StartupUpdatePromptStart,
    StartupUpdatePromptOpenChangelog,
    StartupUpdatePromptDismiss,
    LibraryKey(KeyEvent),
    SearchKey(KeyEvent),
    SettingsKey(KeyEvent),
    GameDetailKey(KeyEvent),
    ExtrasPickerKey(KeyEvent),
    MetadataMatchKey(KeyEvent),
    SetupWizardKey(KeyEvent),
    Background(BackgroundAction),
}

/// Map a key press to zero or more actions (global overlays, chords, then screen dispatch).
pub(crate) fn map_key_to_actions(app: &App, key: &KeyEvent) -> Vec<Action> {
    if key.kind != KeyEventKind::Press {
        return Vec::new();
    }

    if app.global_error.is_some() || app.global_notice.is_some() {
        if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
            return vec![Action::DismissGlobalMessage];
        }
        return Vec::new();
    }

    if app.startup_splash.is_some() {
        return vec![Action::DismissStartupSplash];
    }

    if let Some(ref prompt) = app.startup_update_prompt {
        if prompt.updating {
            return Vec::new();
        }
        return map_startup_update_prompt_key(key);
    }

    if app.show_keyboard_help {
        return match map_keyboard_help_key(key.code) {
            KeyboardHelpInput::Close => vec![Action::HideKeyboardHelp],
            KeyboardHelpInput::Ignore => Vec::new(),
            input => vec![Action::KeyboardHelpInput(input)],
        };
    }

    if key.code == KeyCode::F(1) {
        return vec![Action::ShowKeyboardHelp];
    }
    if key.code == KeyCode::Char('?') && app.allows_global_question_help() {
        return vec![Action::ShowKeyboardHelp];
    }

    if key.code == KeyCode::Char('d') && !app.blocks_global_d_shortcut() {
        return vec![Action::ToggleDownloadOverlay];
    }

    if key.code == KeyCode::Char('/') && !app.blocks_global_slash_shortcut() {
        return vec![Action::ToggleSearchOverlay];
    }

    if key.code == KeyCode::Char(',') && !app.blocks_global_comma_shortcut() {
        return vec![Action::ToggleSettingsOverlay];
    }

    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('r') | KeyCode::Char('R'))
        && !app.blocks_global_chord_shortcuts()
    {
        return vec![Action::RescanLibrary(ScanCacheInvalidate::AllPlatforms)];
    }

    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('u') | KeyCode::Char('U'))
        && !app.blocks_global_chord_shortcuts()
    {
        return vec![Action::ToggleLibraryUploadPrompt];
    }

    super::handlers::screen_keys::map_screen_key(app, key)
}

fn map_startup_update_prompt_key(key: &KeyEvent) -> Vec<Action> {
    match key.code {
        KeyCode::Char('u')
        | KeyCode::Char('U')
        | KeyCode::Char('y')
        | KeyCode::Char('Y')
        | KeyCode::Enter => vec![Action::StartupUpdatePromptStart],
        KeyCode::Char('c') | KeyCode::Char('C') => vec![Action::StartupUpdatePromptOpenChangelog],
        KeyCode::Esc
        | KeyCode::Char('s')
        | KeyCode::Char('S')
        | KeyCode::Char('n')
        | KeyCode::Char('N')
        | KeyCode::Char('q')
        | KeyCode::Char('Q') => vec![Action::StartupUpdatePromptDismiss],
        _ => Vec::new(),
    }
}

impl App {
    /// Convert an [`AppEvent`] into zero or more [`Action`]s.
    pub(crate) fn map_event(&self, event: AppEvent) -> Vec<Action> {
        match event {
            AppEvent::Key(key) => {
                if Self::is_force_quit_key(&key) {
                    vec![Action::Quit]
                } else {
                    map_key_to_actions(self, &key)
                }
            }
            AppEvent::Tick => vec![Action::ProcessDeferredRomLoad],
            AppEvent::Paste(_) => Vec::new(),
            AppEvent::Background(bg) => vec![Action::Background(bg)],
            AppEvent::AutoDismissSplash => vec![Action::DismissStartupSplash],
        }
    }
}
