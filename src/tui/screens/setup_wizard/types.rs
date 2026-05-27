//! Setup wizard step and state types.

use crate::tui::path_picker::PathPicker;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthKind {
    Pairing,
    Basic,
    Bearer,
    ApiKey,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Url,
    Https,
    Download,
    CustomConsolePaths,
    AuthMenu,
    BasicUser,
    BasicPass,
    Bearer,
    ApiHeader,
    ApiKey,
    PairingCode,
    Summary,
}

/// Interactive setup run before the main TUI when `API_BASE_URL` is missing.
pub struct SetupWizard {
    pub(crate) step: Step,
    pub(crate) auth_kind: AuthKind,
    pub(crate) auth_menu_selected: usize,
    pub(crate) url: String,
    pub(crate) url_cursor: usize,
    pub(crate) download_picker: PathPicker,
    pub(crate) username: String,
    pub(crate) user_cursor: usize,
    pub(crate) password: String,
    pub(crate) bearer_token: String,
    pub(crate) bearer_cursor: usize,
    pub(crate) api_header: String,
    pub(crate) header_cursor: usize,
    pub(crate) api_key: String,
    pub(crate) api_key_cursor: usize,
    pub(crate) pairing_code: String,
    pub(crate) pairing_cursor: usize,
    /// Empty field + `true` means resolve secret from OS keyring on save (disk had `<stored-in-keyring>`).
    pub(crate) reuse_keyring_password: bool,
    pub(crate) reuse_keyring_bearer: bool,
    pub(crate) reuse_keyring_api_key: bool,
    pub testing: bool,
    pub use_https: bool,
    pub(crate) skip_custom_console_paths: bool,
    pub error: Option<String>,
}
