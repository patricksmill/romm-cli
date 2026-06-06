//! Configuration and authentication for the ROMM client.
//!
//! This module is deliberately independent of any particular frontend:
//! both the TUI and the command-line subcommands share the same `Config`
//! and `AuthConfig` types.
//!
//! ## Configuration precedence
//!
//! Configuration is layered **per field**. Lowest precedence wins first; higher layers override
//! only the fields they set. There is **no** automatic `.env` file loading.
//!
//! ### Layer 1 — built-in defaults
//!
//! Examples: `use_https = true`, theme `terminal`, OS download directory when unset.
//!
//! ### Layer 2 — `config.json`
//!
//! User file at [`user_config_json_path`], written by `romm-cli init`, the TUI setup wizard,
//! Settings, or `auth login`. Fills fields not set by higher layers.
//!
//! ### Layer 3 — process environment
//!
//! Overrides `config.json` per field in [`load_config`]. Includes `API_BASE_URL`,
//! `ROMM_ROMS_DIR` / `ROMM_DOWNLOAD_DIR`, `API_USE_HTTPS`, auth vars (`API_TOKEN`,
//! `ROMM_TOKEN_FILE` / `API_TOKEN_FILE`, etc.), `ROMM_THEME`, and path overrides
//! (`ROMM_CACHE_PATH`, `ROMM_OPENAPI_PATH`, …).
//!
//! ### Layer 4 — OS keyring
//!
//! After env + JSON merge, secrets that are still placeholders (including
//! [`KEYRING_SECRET_PLACEHOLDER`]) are resolved via the OS keyring (`keyring` crate, service
//! `romm-cli`). On Windows the stored credential target is typically
//! `API_TOKEN.romm-cli`, `API_PASSWORD.romm-cli`, or `API_KEY.romm-cli`. Missing entries are
//! silent; other keyring errors are logged at warn (never with secret values). On save, a
//! successful store is followed by read-back verification before writing the sentinel to JSON.
//!
//! ### Layer 5 — command-specific CLI (runtime only)
//!
//! Narrow exceptions on top of [`load_config`] for a single invocation (not global):
//!
//! | Command / flag | Effect |
//! |----------------|--------|
//! | `download -o` / `--output` | Download directory for that run; ignores env and file |
//! | `sync run --download-dir` | Save download base; defaults to manifest parent, not `save_sync.save_dir` |
//! | `init` / `auth login` flags | **Persist to `config.json`**; effective on the next run unless env overrides |
//!
//! There are no global `--url` / `--token` flags on normal API commands; connection settings
//! come from env + file + keyring via [`load_config`].

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use keyring_core::{Entry, Error as KeyringError, Result as KeyringResult};

use crate::error::{ConfigError, DownloadError};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Supported authentication modes for the RomM API.
#[derive(Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// Basic authentication with username and password.
    Basic {
        /// The RomM username.
        username: String,
        /// The RomM password.
        password: String,
    },
    /// Bearer token authentication (the standard for most API interactions).
    Bearer {
        /// The raw bearer token string.
        token: String,
    },
    /// API Key authentication via a custom HTTP header.
    ApiKey {
        /// Name of the HTTP header (e.g., "X-Api-Key").
        header: String,
        /// The API key value.
        key: String,
    },
}

impl std::fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const REDACTED: &str = "<redacted>";
        match self {
            Self::Basic { username, .. } => f
                .debug_struct("Basic")
                .field("username", username)
                .field("password", &REDACTED)
                .finish(),
            Self::Bearer { .. } => f.debug_struct("Bearer").field("token", &REDACTED).finish(),
            Self::ApiKey { header, .. } => f
                .debug_struct("ApiKey")
                .field("header", header)
                .field("key", &REDACTED)
                .finish(),
        }
    }
}

/// Default checked state for categories in the TUI extras picker (when each row exists).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtrasDefaults {
    /// Pre-check related ROM rows (updates/DLC) when opening the extras picker.
    pub include_related_roms: bool,
    /// Pre-check cover when `url_cover` is set.
    pub include_cover: bool,
    /// Pre-check manual when `url_manual` is set.
    pub include_manual: bool,
}

impl Default for ExtrasDefaults {
    fn default() -> Self {
        Self {
            include_related_roms: true,
            include_cover: true,
            include_manual: true,
        }
    }
}

/// Default library browse split: left pane width as a percentage.
pub const LIBRARY_LEFT_PANEL_PERCENT_DEFAULT: u16 = 30;
/// Minimum library left pane width (percent).
pub const LIBRARY_LEFT_PANEL_PERCENT_MIN: u16 = 15;
/// Maximum library left pane width (percent).
pub const LIBRARY_LEFT_PANEL_PERCENT_MAX: u16 = 50;

/// Default game detail cover column width in terminal cells.
pub const GAME_DETAIL_COVER_PANEL_WIDTH_DEFAULT: u16 = 42;
/// Minimum game detail cover column width.
pub const GAME_DETAIL_COVER_PANEL_WIDTH_MIN: u16 = 20;
/// Maximum game detail cover column width.
pub const GAME_DETAIL_COVER_PANEL_WIDTH_MAX: u16 = 60;

fn default_library_left_panel_percent() -> u16 {
    LIBRARY_LEFT_PANEL_PERCENT_DEFAULT
}

fn default_game_detail_cover_panel_width() -> u16 {
    GAME_DETAIL_COVER_PANEL_WIDTH_DEFAULT
}

/// TUI panel layout preferences (library split and game detail cover width).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TuiLayoutConfig {
    /// Library browse: consoles/collections pane width as a percentage.
    #[serde(default = "default_library_left_panel_percent")]
    pub library_left_panel_percent: u16,
    /// Game detail: cover column width in terminal cells.
    #[serde(default = "default_game_detail_cover_panel_width")]
    pub game_detail_cover_panel_width: u16,
}

impl Default for TuiLayoutConfig {
    fn default() -> Self {
        Self {
            library_left_panel_percent: LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
            game_detail_cover_panel_width: GAME_DETAIL_COVER_PANEL_WIDTH_DEFAULT,
        }
    }
}

impl TuiLayoutConfig {
    /// Clamp values to supported bounds (e.g. after manual JSON edits).
    pub fn normalized(self) -> Self {
        Self {
            library_left_panel_percent: self.library_left_panel_percent.clamp(
                LIBRARY_LEFT_PANEL_PERCENT_MIN,
                LIBRARY_LEFT_PANEL_PERCENT_MAX,
            ),
            game_detail_cover_panel_width: self.game_detail_cover_panel_width.clamp(
                GAME_DETAIL_COVER_PANEL_WIDTH_MIN,
                GAME_DETAIL_COVER_PANEL_WIDTH_MAX,
            ),
        }
    }
}

/// Legacy `roms_layout.mode` values accepted when reading older configs.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum LegacyRomsLayoutMode {
    #[default]
    Auto,
    Manual,
}

/// Per-console ROM storage layout preferences.
///
/// Each platform defaults to `{download_dir}/{platform-slug}/`. Entries in
/// [`platform_dirs`](Self::platform_dirs) override that with an absolute custom path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RomsLayoutConfig {
    /// Accepted when reading legacy configs; never written.
    #[serde(default, skip_serializing, rename = "mode")]
    _legacy_mode: Option<LegacyRomsLayoutMode>,
    /// Platform id → absolute custom directory path.
    #[serde(default)]
    pub platform_dirs: HashMap<u64, String>,
}

/// Save sync preferences shared by CLI/TUI frontends.
///
/// Each platform defaults to `{save_base}/{platform-slug}/`. Entries in
/// [`platform_dirs`](Self::platform_dirs) override that with an absolute custom path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SaveSyncConfig {
    /// Local folder used by TUI save download/sync flows.
    #[serde(default)]
    pub save_dir: Option<String>,
    /// RomM sync device id used by manual push-pull.
    #[serde(default)]
    pub device_id: Option<String>,
    /// Platform id → absolute custom save directory path.
    #[serde(default)]
    pub platform_dirs: HashMap<u64, String>,
}

/// High-level configuration for the `romm-cli` application.
///
/// This struct holds the connection details and authentication settings
/// required to communicate with a RomM server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The base URL (origin) of the RomM server (e.g., "<https://romm.example.com>").
    pub base_url: String,
    /// Local directory where ROMs should be downloaded.
    pub download_dir: String,
    /// Whether to force HTTPS for API calls.
    pub use_https: bool,
    /// Active authentication configuration, if any.
    pub auth: Option<AuthConfig>,
    /// TUI extras picker: which categories start checked when rows exist.
    #[serde(default)]
    pub extras_defaults: ExtrasDefaults,
    /// TUI save-management settings.
    #[serde(default)]
    pub save_sync: SaveSyncConfig,
    /// Optional per-console custom directory overrides.
    #[serde(default)]
    pub roms_layout: RomsLayoutConfig,
    /// TUI color theme ID (see ratatui-themekit `available_theme_ids`).
    #[serde(default = "default_theme_id")]
    pub theme: String,
    /// TUI panel layout (library split, game detail cover width).
    #[serde(default)]
    pub tui_layout: TuiLayoutConfig,
}

/// Default TUI theme ID when none is configured.
pub const DEFAULT_THEME_ID: &str = "terminal";

pub fn default_theme_id() -> String {
    DEFAULT_THEME_ID.to_string()
}

pub fn resolved_save_dir(config: &Config) -> PathBuf {
    config
        .save_sync
        .save_dir
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&config.download_dir).join("saves"))
}

/// Resolve the directory where save files for a console should be stored.
pub fn resolve_console_save_dir(
    save_sync: &SaveSyncConfig,
    base_save_dir: &std::path::Path,
    platform_id: u64,
    platform_fs_slug: Option<&str>,
    platform_slug: Option<&str>,
) -> Result<PathBuf, DownloadError> {
    crate::core::download::resolve_console_save_dir(
        save_sync,
        base_save_dir,
        platform_id,
        platform_fs_slug,
        platform_slug,
    )
}

/// Resolve the directory where a specific game's saves should be downloaded.
pub fn resolve_game_save_dir(
    config: &Config,
    rom: &crate::types::Rom,
) -> Result<PathBuf, DownloadError> {
    crate::core::download::resolve_game_save_dir(config, rom)
}

fn is_placeholder(value: &str) -> bool {
    value.contains("your-") || value.contains("placeholder") || value.trim().is_empty()
}

/// Written to `config.json` when the real secret is stored in the OS keyring (`persist_user_config`).
pub const KEYRING_SECRET_PLACEHOLDER: &str = "<stored-in-keyring>";

/// Returns true if `s` is the sentinel written to disk when the secret lives in the keyring.
pub fn is_keyring_placeholder(s: &str) -> bool {
    s == KEYRING_SECRET_PLACEHOLDER
}

/// Normalizes a RomM site URL by trimming whitespace, trailing slashes, and removing the `/api` suffix.
///
/// # Examples
///
/// ```
/// # use romm_cli::config::normalize_romm_origin;
/// assert_eq!(normalize_romm_origin("http://localhost:8080/api/"), "http://localhost:8080");
/// assert_eq!(normalize_romm_origin(" https://romm.example.com "), "https://romm.example.com");
/// ```
pub fn normalize_romm_origin(url: &str) -> String {
    let mut s = url.trim().trim_end_matches('/').to_string();
    if s.ends_with("/api") {
        s.truncate(s.len() - 4);
    }
    s.trim_end_matches('/').to_string()
}

// ---------------------------------------------------------------------------
// Keyring helpers
// ---------------------------------------------------------------------------

const KEYRING_SERVICE: &str = "romm-cli";

/// Store a secret in the OS keyring under the `romm-cli` service name.
///
/// This is used to securely persist passwords, tokens, and API keys without
/// writing them in plaintext to `config.json`.
pub fn keyring_store(key: &str, value: &str) -> Result<(), ConfigError> {
    let entry = Entry::new(KEYRING_SERVICE, key).map_err(|e| ConfigError::KeyringEntry {
        key: key.to_string(),
        message: e.to_string(),
    })?;
    entry
        .set_password(value)
        .map_err(|e| ConfigError::KeyringStore {
            key: key.to_string(),
            message: e.to_string(),
        })
}

/// Map `get_password` result: [`keyring::Error::NoEntry`] is normal when no credential exists (no log).
/// Other errors are logged (never logs secret bytes).
fn keyring_get_password_result(key: &str, result: KeyringResult<String>) -> Option<String> {
    match result {
        Ok(s) => Some(s),
        Err(KeyringError::NoEntry) => None,
        Err(e) => {
            tracing::warn!("keyring get_password for key {key}: {e}");
            None
        }
    }
}

/// Retrieve a secret from the OS keyring, returning `None` if not found or on error.
///
/// Unexpected errors are logged at the `warn` level.
pub(crate) fn keyring_get(key: &str) -> Option<String> {
    let entry = match Entry::new(KEYRING_SERVICE, key) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("keyring Entry::new for key {key}: {e}");
            return None;
        }
    };
    keyring_get_password_result(key, entry.get_password())
}

/// After a successful `set_password`, confirm read-back matches `expected`.
/// If not, the caller should keep plaintext in JSON to avoid data loss.
fn keyring_verify_read_back_matches(key: &str, expected: &str) -> bool {
    let entry = match Entry::new(KEYRING_SERVICE, key) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(
                "keyring verify: Entry::new for key {key} after successful store: {e}; writing plaintext to config.json"
            );
            return false;
        }
    };
    match entry.get_password() {
        Ok(read) if read == expected => true,
        Ok(_) => {
            tracing::warn!(
                "keyring verify: read-back for key {key} did not match; writing plaintext to config.json"
            );
            false
        }
        Err(e) => {
            tracing::warn!(
                "keyring verify: get_password for key {key} after successful store: {e}; writing plaintext to config.json"
            );
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

/// Directory for user-level config (`romm-cli` under the OS config dir).
pub fn user_config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("ROMM_TEST_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    dirs::config_dir().map(|d| d.join("romm-cli"))
}

/// Path to the user-level `config.json` file (`.../romm-cli/config.json`).
pub fn user_config_json_path() -> Option<PathBuf> {
    user_config_dir().map(|d| d.join("config.json"))
}

/// Reads `config.json` from disk only (no env merge, no keyring resolution).
/// Used by the TUI setup wizard to detect `<stored-in-keyring>` placeholders.
pub fn read_user_config_json_from_disk() -> Option<Config> {
    let path = user_config_json_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Auth to pass to [`persist_user_config`] when saving non-auth fields (e.g. TUI Settings).
///
/// Prefer the in-memory [`Config::auth`]. If it is `None` (e.g. [`load_config`] could not read the
/// token from the keyring), reuse `auth` from [`read_user_config_json_from_disk`] so we do not
/// overwrite `config.json` with `"auth": null` while the file still held a bearer sentinel.
pub fn auth_for_persist_merge(in_memory: Option<AuthConfig>) -> Option<AuthConfig> {
    in_memory.or_else(|| read_user_config_json_from_disk().and_then(|c| c.auth))
}

/// Where the OpenAPI spec is cached (`.../romm-cli/openapi.json`).
///
/// Override with `ROMM_OPENAPI_PATH` (absolute or relative path).
pub fn openapi_cache_path() -> Result<PathBuf, ConfigError> {
    if let Ok(p) = std::env::var("ROMM_OPENAPI_PATH") {
        return Ok(PathBuf::from(p));
    }
    let dir = user_config_dir().ok_or(ConfigError::ConfigDirUnavailable)?;
    Ok(dir.join("openapi.json"))
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.trim().is_empty())
}

/// Returns true if the application should check for updates on startup.
///
/// This is controlled by the `ROMM_CHECK_UPDATES` environment variable.
/// Defaults to `true`.
pub fn should_check_updates() -> bool {
    match std::env::var("ROMM_CHECK_UPDATES") {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            !matches!(normalized.as_str(), "0" | "false" | "no" | "off")
        }
        Err(_) => true,
    }
}

/// Max bytes read from bearer token files (`ROMM_TOKEN_FILE` / `API_TOKEN_FILE`).
const MAX_TOKEN_FILE_BYTES: usize = 64 * 1024;

/// Bearer token: `API_TOKEN` env, else UTF-8 file at `ROMM_TOKEN_FILE` or `API_TOKEN_FILE` path.
fn token_from_env_or_file() -> Result<Option<String>, ConfigError> {
    if let Some(t) = env_nonempty("API_TOKEN") {
        return Ok(Some(t));
    }
    let path = env_nonempty("ROMM_TOKEN_FILE").or_else(|| env_nonempty("API_TOKEN_FILE"));
    let Some(path) = path else {
        return Ok(None);
    };
    let path = path.trim();
    let bytes = fs::read(path).map_err(|e| ConfigError::TokenFileRead {
        path: path.to_string(),
        source: e,
    })?;
    if bytes.len() > MAX_TOKEN_FILE_BYTES {
        return Err(ConfigError::TokenFileTooLarge {
            max: MAX_TOKEN_FILE_BYTES,
        });
    }
    let s = String::from_utf8(bytes).map_err(|_| ConfigError::TokenFileInvalidUtf8 {
        path: path.to_string(),
    })?;
    let t = s.trim();
    if t.is_empty() {
        return Err(ConfigError::TokenFileEmpty {
            path: path.to_string(),
        });
    }
    Ok(Some(t.to_string()))
}

/// Returns true when [`load_config`] has no resolved [`AuthConfig::Bearer`] (etc.) but `config.json`
/// on disk still contains [`KEYRING_SECRET_PLACEHOLDER`] (OS keyring could not supply the secret).
pub fn disk_has_unresolved_keyring_sentinel(config: &Config) -> bool {
    if config.auth.is_some() {
        return false;
    }
    let Some(disk) = read_user_config_json_from_disk() else {
        return false;
    };
    match &disk.auth {
        Some(AuthConfig::Bearer { token }) => is_keyring_placeholder(token),
        Some(AuthConfig::Basic { password, .. }) => is_keyring_placeholder(password),
        Some(AuthConfig::ApiKey { key, .. }) => is_keyring_placeholder(key),
        None => false,
    }
}

/// Loads the merged configuration from the process environment, `config.json`, and the OS keyring.
///
/// This function handles the precedence of configuration sources:
/// 1. Environment variables (highest priority).
/// 2. `config.json` file.
/// 3. OS Keyring (for secrets).
///
/// # Errors
///
/// Returns an error if `API_BASE_URL` is not set or if there are issues reading token files.
pub fn load_config() -> Result<Config, ConfigError> {
    // 1. Load from JSON first (if it exists)
    let mut json_config = None;
    if let Some(path) = user_config_json_path() {
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<Config>(&content) {
                    json_config = Some(config);
                }
            }
        }
    }

    // 2. Resolve base_url
    let base_raw = env_nonempty("API_BASE_URL")
        .or_else(|| json_config.as_ref().map(|c| c.base_url.clone()))
        .ok_or(ConfigError::MissingBaseUrl)?;
    let mut base_url = normalize_romm_origin(&base_raw);

    // 3. Resolve ROM storage directory
    let download_dir = env_nonempty("ROMM_ROMS_DIR")
        .or_else(|| env_nonempty("ROMM_DOWNLOAD_DIR"))
        .or_else(|| json_config.as_ref().map(|c| c.download_dir.clone()))
        .unwrap_or_else(|| {
            dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
                .join("romm-cli")
                .display()
                .to_string()
        });

    // 4. Resolve use_https
    let use_https = if let Ok(s) = std::env::var("API_USE_HTTPS") {
        s.to_lowercase() == "true"
    } else if let Some(c) = &json_config {
        c.use_https
    } else {
        true
    };

    if use_https && base_url.starts_with("http://") {
        base_url = base_url.replace("http://", "https://");
    }

    // 5. Resolve Auth
    let mut username = env_nonempty("API_USERNAME");
    let mut password = env_nonempty("API_PASSWORD");
    let mut token = token_from_env_or_file()?;
    let mut api_key = env_nonempty("API_KEY");
    let mut api_key_header = env_nonempty("API_KEY_HEADER");

    if let Some(c) = &json_config {
        if let Some(auth) = &c.auth {
            match auth {
                AuthConfig::Basic {
                    username: u,
                    password: p,
                } => {
                    if username.is_none() {
                        username = Some(u.clone());
                    }
                    if password.is_none() {
                        password = Some(p.clone());
                    }
                }
                AuthConfig::Bearer { token: t } => {
                    if token.is_none() {
                        token = Some(t.clone());
                    }
                }
                AuthConfig::ApiKey { header: h, key: k } => {
                    if api_key_header.is_none() {
                        api_key_header = Some(h.clone());
                    }
                    if api_key.is_none() {
                        api_key = Some(k.clone());
                    }
                }
            }
        }
    }

    // Resolve placeholders from keyring (including disk sentinel `<stored-in-keyring>`).
    if let Some(p) = &password {
        if is_placeholder(p) || is_keyring_placeholder(p) {
            if let Some(k) = keyring_get("API_PASSWORD") {
                password = Some(k);
            }
        }
    } else {
        password = keyring_get("API_PASSWORD");
    }

    if let Some(t) = &token {
        if is_placeholder(t) || is_keyring_placeholder(t) {
            if let Some(k) = keyring_get("API_TOKEN") {
                token = Some(k);
            }
        }
    } else {
        token = keyring_get("API_TOKEN");
    }

    if let Some(k) = &api_key {
        if is_placeholder(k) || is_keyring_placeholder(k) {
            if let Some(kr) = keyring_get("API_KEY") {
                api_key = Some(kr);
            }
        }
    } else {
        api_key = keyring_get("API_KEY");
    }

    if let Some(ref p) = password {
        if is_keyring_placeholder(p) {
            tracing::warn!(
                "Could not read API_PASSWORD from the OS keyring; value is still <stored-in-keyring>. \
                 On Windows, look for a Generic credential with target API_PASSWORD.romm-cli."
            );
        }
    }
    if let Some(ref t) = token {
        if is_keyring_placeholder(t) {
            tracing::warn!(
                "Could not read API_TOKEN from the OS keyring; value is still <stored-in-keyring>. \
                 On Windows, look for a Generic credential with target API_TOKEN.romm-cli."
            );
        }
    }
    if let Some(ref k) = api_key {
        if is_keyring_placeholder(k) {
            tracing::warn!(
                "Could not read API_KEY from the OS keyring; value is still <stored-in-keyring>. \
                 On Windows, look for a Generic credential with target API_KEY.romm-cli."
            );
        }
    }

    let auth = if let (Some(user), Some(pass)) = (username, password) {
        if !is_placeholder(&pass) && !is_keyring_placeholder(&pass) {
            Some(AuthConfig::Basic {
                username: user,
                password: pass,
            })
        } else {
            None
        }
    } else if let (Some(key), Some(header)) = (api_key, api_key_header) {
        if !is_placeholder(&key) && !is_keyring_placeholder(&key) {
            Some(AuthConfig::ApiKey { header, key })
        } else {
            None
        }
    } else if let Some(tok) = token {
        if !is_placeholder(&tok) && !is_keyring_placeholder(&tok) {
            Some(AuthConfig::Bearer { token: tok })
        } else {
            None
        }
    } else {
        None
    };

    let extras_defaults = json_config
        .as_ref()
        .map(|c| c.extras_defaults.clone())
        .unwrap_or_default();
    let save_sync = json_config
        .as_ref()
        .map(|c| c.save_sync.clone())
        .unwrap_or_default();

    let roms_layout = json_config
        .as_ref()
        .map(|c| c.roms_layout.clone())
        .unwrap_or_default();

    let theme = env_nonempty("ROMM_THEME")
        .or_else(|| json_config.as_ref().map(|c| c.theme.clone()))
        .unwrap_or_else(default_theme_id);

    let tui_layout = json_config
        .as_ref()
        .map(|c| c.tui_layout.clone().normalized())
        .unwrap_or_default();

    Ok(Config {
        base_url,
        download_dir,
        use_https,
        auth,
        extras_defaults,
        save_sync,
        roms_layout,
        theme,
        tui_layout,
    })
}

/// Persists the user configuration to `config.json` and stores secrets in the OS keyring.
///
/// This function will:
/// 1. Create the configuration directory if it doesn't exist.
/// 2. Store secrets (password, token, API key) in the OS keyring.
/// 3. Write non-secret configuration to `config.json`.
/// 4. On Unix, set file permissions to 0600 (owner read/write only).
///
/// If a secret cannot be stored in the keyring, it is written in plaintext to `config.json`
/// as a fallback, and a warning is logged.
pub fn persist_user_config(config: &Config) -> Result<(), ConfigError> {
    let Some(path) = user_config_json_path() else {
        return Err(ConfigError::ConfigDirNotFound);
    };
    let dir = path.parent().ok_or(ConfigError::InvalidConfigPath)?;
    std::fs::create_dir_all(dir).map_err(|e| ConfigError::Io {
        context: format!("create {}", dir.display()),
        source: e,
    })?;

    let mut config_to_save = config.clone();

    match &mut config_to_save.auth {
        None => {}
        Some(AuthConfig::Basic { password, .. }) => {
            if is_keyring_placeholder(password) {
                tracing::debug!(
                    "skip keyring store for API_PASSWORD: value is keyring sentinel; leaving disk sentinel unchanged"
                );
            } else if let Err(e) = keyring_store("API_PASSWORD", password) {
                tracing::warn!("keyring store API_PASSWORD: {e}; writing plaintext to config.json");
            } else if keyring_verify_read_back_matches("API_PASSWORD", password.as_str()) {
                *password = KEYRING_SECRET_PLACEHOLDER.to_string();
            }
        }
        Some(AuthConfig::Bearer { token }) => {
            if is_keyring_placeholder(token) {
                tracing::debug!(
                    "skip keyring store for API_TOKEN: value is keyring sentinel; leaving disk sentinel unchanged"
                );
            } else if let Err(e) = keyring_store("API_TOKEN", token) {
                tracing::warn!("keyring store API_TOKEN: {e}; writing plaintext to config.json");
            } else if keyring_verify_read_back_matches("API_TOKEN", token.as_str()) {
                *token = KEYRING_SECRET_PLACEHOLDER.to_string();
            }
        }
        Some(AuthConfig::ApiKey { key, .. }) => {
            if is_keyring_placeholder(key) {
                tracing::debug!(
                    "skip keyring store for API_KEY: value is keyring sentinel; leaving disk sentinel unchanged"
                );
            } else if let Err(e) = keyring_store("API_KEY", key) {
                tracing::warn!("keyring store API_KEY: {e}; writing plaintext to config.json");
            } else if keyring_verify_read_back_matches("API_KEY", key.as_str()) {
                *key = KEYRING_SECRET_PLACEHOLDER.to_string();
            }
        }
    }

    let content = serde_json::to_string_pretty(&config_to_save)?;
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&path).map_err(|e| ConfigError::Io {
            context: format!("write {}", path.display()),
            source: e,
        })?;
        f.write_all(content.as_bytes())
            .map_err(|e| ConfigError::Io {
                context: format!("write {}", path.display()),
                source: e,
            })?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)
            .map_err(|e| ConfigError::Io {
                context: format!("chmod metadata {}", path.display()),
                source: e,
            })?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms).map_err(|e| ConfigError::Io {
            context: format!("chmod {}", path.display()),
            source: e,
        })?;
    }

    Ok(())
}

/// Deletes the config.json file and clears the secrets from the OS keyring.
pub fn reset_all_settings() -> Result<(), ConfigError> {
    if let Some(path) = user_config_json_path() {
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
    for key in ["API_PASSWORD", "API_TOKEN", "API_KEY"] {
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, key) {
            let _ = entry.delete_credential();
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn test_env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::MutexGuard;

    #[test]
    fn keyring_get_password_result_ok() {
        assert_eq!(
            super::keyring_get_password_result("API_TOKEN", Ok("secret".into())),
            Some("secret".into())
        );
    }

    #[test]
    fn keyring_get_password_result_no_entry_is_none() {
        assert_eq!(
            super::keyring_get_password_result("API_TOKEN", Err(KeyringError::NoEntry)),
            None
        );
    }

    #[test]
    fn auth_config_debug_redacts_secrets() {
        let basic = AuthConfig::Basic {
            username: "alice".to_string(),
            password: "sekrit".to_string(),
        };
        let bearer = AuthConfig::Bearer {
            token: "tok123".to_string(),
        };
        let api_key = AuthConfig::ApiKey {
            header: "X-Api-Key".to_string(),
            key: "key456".to_string(),
        };
        let basic_dbg = format!("{basic:?}");
        let bearer_dbg = format!("{bearer:?}");
        let api_key_dbg = format!("{api_key:?}");
        assert!(!basic_dbg.contains("sekrit"));
        assert!(basic_dbg.contains("alice"));
        assert!(!bearer_dbg.contains("tok123"));
        assert!(!api_key_dbg.contains("key456"));
        assert!(api_key_dbg.contains("X-Api-Key"));
    }

    struct TestEnv {
        _guard: MutexGuard<'static, ()>,
        config_dir: PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            let guard = super::test_env_lock()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            clear_auth_env();

            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let config_dir = std::env::temp_dir().join(format!("romm-config-test-{ts}"));
            std::fs::create_dir_all(&config_dir).unwrap();
            std::env::set_var("ROMM_TEST_CONFIG_DIR", &config_dir);

            Self {
                _guard: guard,
                config_dir,
            }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            clear_auth_env();
            std::env::remove_var("ROMM_TEST_CONFIG_DIR");
            let _ = std::fs::remove_dir_all(&self.config_dir);
        }
    }

    #[test]
    fn config_theme_defaults_to_terminal() {
        let cfg: Config = serde_json::from_str(
            r#"{"base_url":"http://x","download_dir":"/tmp","use_https":false}"#,
        )
        .unwrap();
        assert_eq!(cfg.theme, "terminal");
    }

    #[test]
    fn config_theme_round_trip() {
        let json =
            r#"{"base_url":"http://x","download_dir":"/tmp","use_https":false,"theme":"dracula"}"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.theme, "dracula");
    }

    #[test]
    fn config_tui_layout_defaults_when_missing() {
        let cfg: Config = serde_json::from_str(
            r#"{"base_url":"http://x","download_dir":"/tmp","use_https":false}"#,
        )
        .unwrap();
        assert_eq!(cfg.tui_layout, TuiLayoutConfig::default());
    }

    #[test]
    fn config_tui_layout_normalizes_out_of_range_values() {
        let cfg = TuiLayoutConfig {
            library_left_panel_percent: 5,
            game_detail_cover_panel_width: 999,
        }
        .normalized();
        assert_eq!(
            cfg.library_left_panel_percent,
            LIBRARY_LEFT_PANEL_PERCENT_MIN
        );
        assert_eq!(
            cfg.game_detail_cover_panel_width,
            GAME_DETAIL_COVER_PANEL_WIDTH_MAX
        );
    }

    fn clear_auth_env() {
        for key in [
            "API_BASE_URL",
            "ROMM_ROMS_DIR",
            "API_USERNAME",
            "API_PASSWORD",
            "API_TOKEN",
            "ROMM_TOKEN_FILE",
            "API_TOKEN_FILE",
            "API_KEY",
            "API_KEY_HEADER",
            "API_USE_HTTPS",
            "ROMM_THEME",
            "ROMM_TEST_CONFIG_DIR",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn prefers_basic_auth_over_other_modes() {
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_USERNAME", "user");
        std::env::set_var("API_PASSWORD", "pass");
        std::env::set_var("API_TOKEN", "token");
        std::env::set_var("API_KEY", "apikey");
        std::env::set_var("API_KEY_HEADER", "X-Api-Key");

        let cfg = load_config().expect("config should load");
        match cfg.auth {
            Some(AuthConfig::Basic { username, password }) => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("expected basic auth"),
        }
    }

    #[test]
    fn uses_api_key_header_when_token_missing() {
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_KEY", "real-key");
        std::env::set_var("API_KEY_HEADER", "X-Api-Key");

        let cfg = load_config().expect("config should load");
        match cfg.auth {
            Some(AuthConfig::ApiKey { header, key }) => {
                assert_eq!(header, "X-Api-Key");
                assert_eq!(key, "real-key");
            }
            _ => panic!("expected api key auth"),
        }
    }

    #[test]
    fn normalizes_api_base_url_and_enforces_https_by_default() {
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://romm.example/api/");
        let cfg = load_config().expect("config");
        // Upgraded to https by default
        assert_eq!(cfg.base_url, "https://romm.example");
    }

    #[test]
    fn does_not_enforce_https_if_toggle_is_false() {
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://romm.example/api/");
        std::env::set_var("API_USE_HTTPS", "false");
        let cfg = load_config().expect("config");
        assert_eq!(cfg.base_url, "http://romm.example");
    }

    #[test]
    fn normalize_romm_origin_trims_and_strips_api_suffix() {
        assert_eq!(
            normalize_romm_origin("http://localhost:8080/api/"),
            "http://localhost:8080"
        );
        assert_eq!(
            normalize_romm_origin("https://x.example"),
            "https://x.example"
        );
    }

    #[test]
    fn empty_api_username_does_not_enable_basic() {
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_USERNAME", "");
        std::env::set_var("API_PASSWORD", "secret");

        let cfg = load_config().expect("config should load");
        assert!(
            cfg.auth.is_none(),
            "empty API_USERNAME should not pair with password for Basic"
        );
    }

    #[test]
    fn ignores_placeholder_bearer_token() {
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_TOKEN", "your-bearer-token-here");

        let cfg = load_config().expect("config should load");
        assert!(cfg.auth.is_none(), "placeholder token should be ignored");
    }

    #[test]
    fn loads_from_user_json_file() {
        let env = TestEnv::new();
        let config_json = r#"{
            "base_url": "http://from-json-file.test",
            "download_dir": "/tmp/downloads",
            "use_https": false,
            "auth": null
        }"#;

        std::fs::write(env.config_dir.join("config.json"), config_json).unwrap();

        let cfg = load_config().expect("load from user config.json");
        assert_eq!(cfg.base_url, "http://from-json-file.test");
        assert_eq!(cfg.download_dir, "/tmp/downloads");
        assert!(!cfg.use_https);
    }

    #[test]
    fn extras_defaults_default_to_all_true_when_missing_from_json() {
        let config_json = r#"{
            "base_url": "http://from-json-file.test",
            "download_dir": "/tmp/downloads",
            "use_https": false,
            "auth": null
        }"#;
        let cfg: Config = serde_json::from_str(config_json).expect("deserialize legacy config");
        assert!(cfg.extras_defaults.include_related_roms);
        assert!(cfg.extras_defaults.include_cover);
        assert!(cfg.extras_defaults.include_manual);
    }

    #[test]
    fn save_sync_defaults_when_missing_from_legacy_json() {
        let config_json = r#"{
            "base_url": "http://from-json-file.test",
            "download_dir": "/tmp/downloads",
            "use_https": false,
            "auth": null
        }"#;
        let cfg: Config = serde_json::from_str(config_json).expect("deserialize legacy config");
        assert_eq!(cfg.save_sync, SaveSyncConfig::default());
    }

    #[test]
    fn roms_layout_defaults_when_missing_from_legacy_json() {
        let config_json = r#"{
            "base_url": "http://from-json-file.test",
            "download_dir": "/tmp/downloads",
            "use_https": false,
            "auth": null
        }"#;
        let cfg: Config = serde_json::from_str(config_json).expect("deserialize legacy config");
        assert_eq!(cfg.roms_layout, RomsLayoutConfig::default());
    }

    #[test]
    fn roms_layout_deserializes_legacy_mode_with_platform_dirs() {
        let config_json = r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp/downloads",
            "use_https": false,
            "auth": null,
            "roms_layout": {
                "mode": "manual",
                "platform_dirs": {
                    "7": "D:\\Roms\\Switch",
                    "3": "/roms/nes"
                }
            }
        }"#;
        let cfg: Config = serde_json::from_str(config_json).expect("deserialize");
        assert_eq!(
            cfg.roms_layout.platform_dirs.get(&7).map(String::as_str),
            Some("D:\\Roms\\Switch")
        );
        assert_eq!(
            cfg.roms_layout.platform_dirs.get(&3).map(String::as_str),
            Some("/roms/nes")
        );
    }

    #[test]
    fn roms_layout_honors_platform_dirs_without_legacy_mode() {
        let config_json = r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": null,
            "roms_layout": {
                "platform_dirs": { "1": "/custom/nes" }
            }
        }"#;
        let cfg: Config = serde_json::from_str(config_json).expect("deserialize");
        assert_eq!(
            cfg.roms_layout.platform_dirs.get(&1).map(String::as_str),
            Some("/custom/nes")
        );
    }

    #[test]
    fn roms_layout_save_omits_legacy_mode_field() {
        let config_json = r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": null,
            "roms_layout": {
                "mode": "manual",
                "platform_dirs": { "1": "/custom/nes" }
            }
        }"#;
        let cfg: Config = serde_json::from_str(config_json).expect("deserialize");
        let json = serde_json::to_string(&cfg.roms_layout).expect("serialize");
        assert!(!json.contains("mode"));
        assert!(json.contains("platform_dirs"));
    }

    #[test]
    fn resolved_save_dir_falls_back_to_download_dir_saves() {
        let cfg = Config {
            base_url: "http://example.test".into(),
            download_dir: "/roms".into(),
            use_https: false,
            auth: None,
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig::default(),
            roms_layout: RomsLayoutConfig::default(),
            theme: default_theme_id(),
            tui_layout: TuiLayoutConfig::default(),
        };
        assert_eq!(
            resolved_save_dir(&cfg),
            PathBuf::from("/roms").join("saves")
        );
    }

    #[test]
    fn save_sync_deserializes_platform_dirs() {
        let config_json = r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": null,
            "save_sync": {
                "save_dir": "/saves",
                "platform_dirs": {
                    "7": "D:\\Saves\\Switch"
                }
            }
        }"#;
        let cfg: Config = serde_json::from_str(config_json).expect("deserialize");
        assert_eq!(
            cfg.save_sync.platform_dirs.get(&7).map(String::as_str),
            Some("D:\\Saves\\Switch")
        );
    }

    #[test]
    fn save_sync_save_includes_platform_dirs() {
        let cfg = Config {
            base_url: "http://example.test".into(),
            download_dir: "/tmp".into(),
            use_https: false,
            auth: None,
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig {
                save_dir: Some("/saves".into()),
                device_id: None,
                platform_dirs: HashMap::from([(7, "D:\\Saves\\Switch".into())]),
            },
            roms_layout: RomsLayoutConfig::default(),
            theme: default_theme_id(),
            tui_layout: TuiLayoutConfig::default(),
        };
        let json = serde_json::to_string(&cfg.save_sync).expect("serialize");
        assert!(json.contains("platform_dirs"));
    }

    #[test]
    fn roms_dir_env_takes_precedence_over_legacy_download_dir_env() {
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("ROMM_ROMS_DIR", "/preferred-roms");
        std::env::set_var("ROMM_DOWNLOAD_DIR", "/legacy-downloads");

        let cfg = load_config().expect("config should load");
        assert_eq!(cfg.download_dir, "/preferred-roms");
    }

    #[test]
    fn auth_for_persist_merge_prefers_in_memory() {
        let env = TestEnv::new();
        let on_disk = r#"{
            "base_url": "http://disk.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "Bearer": { "token": "from-disk" } }
        }"#;
        std::fs::write(env.config_dir.join("config.json"), on_disk).unwrap();

        let mem = Some(AuthConfig::Bearer {
            token: "from-memory".into(),
        });
        let merged = auth_for_persist_merge(mem.clone());
        assert_eq!(format!("{:?}", merged), format!("{:?}", mem));
    }

    #[test]
    fn auth_for_persist_merge_falls_back_to_disk_when_memory_empty() {
        let env = TestEnv::new();
        let on_disk = r#"{
            "base_url": "http://disk.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "Bearer": { "token": "<stored-in-keyring>" } }
        }"#;
        std::fs::write(env.config_dir.join("config.json"), on_disk).unwrap();

        let merged = auth_for_persist_merge(None);
        match merged {
            Some(AuthConfig::Bearer { token }) => {
                assert_eq!(token, KEYRING_SECRET_PLACEHOLDER);
            }
            _ => panic!("expected bearer auth from disk"),
        }
    }

    #[test]
    fn bearer_keyring_sentinel_without_keyring_entry_yields_no_auth() {
        let env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        let config_json = r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "Bearer": { "token": "<stored-in-keyring>" } }
        }"#;
        std::fs::write(env.config_dir.join("config.json"), config_json).unwrap();

        let cfg = load_config().expect("load");
        assert!(
            cfg.auth.is_none(),
            "unresolved keyring sentinel must not become Bearer auth in Config"
        );
        assert!(disk_has_unresolved_keyring_sentinel(&cfg));
    }

    #[test]
    fn bearer_token_from_romm_token_file() {
        let env = TestEnv::new();
        let token_path = env.config_dir.join("secret.token");
        std::fs::write(&token_path, "  tok-from-file\n").unwrap();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("ROMM_TOKEN_FILE", token_path.to_str().unwrap());

        let cfg = load_config().expect("load");
        match cfg.auth {
            Some(AuthConfig::Bearer { token }) => assert_eq!(token, "tok-from-file"),
            _ => panic!("expected bearer from token file"),
        }
    }

    #[test]
    fn api_token_env_wins_over_token_file() {
        let env = TestEnv::new();
        let token_path = env.config_dir.join("secret.token");
        std::fs::write(&token_path, "from-file").unwrap();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("ROMM_TOKEN_FILE", token_path.to_str().unwrap());
        std::env::set_var("API_TOKEN", "from-env");

        let cfg = load_config().expect("load");
        match cfg.auth {
            Some(AuthConfig::Bearer { token }) => assert_eq!(token, "from-env"),
            _ => panic!("expected env API_TOKEN to win"),
        }
    }

    #[test]
    fn romm_token_file_overrides_json_bearer() {
        let env = TestEnv::new();
        let token_path = env.config_dir.join("secret.token");
        std::fs::write(&token_path, "from-file").unwrap();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("ROMM_TOKEN_FILE", token_path.to_str().unwrap());
        let config_json = r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "Bearer": { "token": "from-json" } }
        }"#;
        std::fs::write(env.config_dir.join("config.json"), config_json).unwrap();

        let cfg = load_config().expect("load");
        match cfg.auth {
            Some(AuthConfig::Bearer { token }) => assert_eq!(token, "from-file"),
            _ => panic!("expected token file to override json"),
        }
    }

    #[test]
    fn romm_token_file_missing_errors() {
        let env = TestEnv::new();
        let missing = env.config_dir.join("this-token-file-does-not-exist");
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("ROMM_TOKEN_FILE", missing.to_str().unwrap());

        let err = load_config().expect_err("missing token file should error");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("read bearer token file"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn romm_token_file_empty_errors() {
        let env = TestEnv::new();
        let token_path = env.config_dir.join("empty.token");
        std::fs::write(&token_path, "   \n\t  ").unwrap();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("ROMM_TOKEN_FILE", token_path.to_str().unwrap());

        let err = load_config().expect_err("empty token file should error");
        assert!(
            format!("{err:#}").contains("empty"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn romm_token_file_too_large_errors() {
        let env = TestEnv::new();
        let token_path = env.config_dir.join("huge.token");
        std::fs::write(&token_path, vec![b'a'; MAX_TOKEN_FILE_BYTES + 1]).unwrap();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("ROMM_TOKEN_FILE", token_path.to_str().unwrap());

        let err = load_config().expect_err("oversized token file should error");
        assert!(
            format!("{err:#}").contains("max size"),
            "unexpected error: {err:#}"
        );
    }

    /// When auth is merged from disk as [`KEYRING_SECRET_PLACEHOLDER`], persist must not call
    /// `keyring_store` with that literal (would overwrite the real vault entry). JSON should still
    /// contain the sentinel and updated non-auth fields.
    #[test]
    fn persist_user_config_preserves_sentinel_secrets_in_json() {
        let env = TestEnv::new();
        let path = env.config_dir.join("config.json");

        persist_user_config(&Config {
            base_url: "https://updated.example".into(),
            download_dir: "/var/romm-dl".into(),
            use_https: true,
            auth: Some(AuthConfig::Bearer {
                token: KEYRING_SECRET_PLACEHOLDER.to_string(),
            }),
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig::default(),
            roms_layout: RomsLayoutConfig::default(),
            theme: default_theme_id(),
            tui_layout: TuiLayoutConfig::default(),
        })
        .expect("persist bearer sentinel");

        let cfg: Config = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(cfg.base_url, "https://updated.example");
        assert_eq!(cfg.download_dir, "/var/romm-dl");
        assert!(cfg.use_https);
        match cfg.auth {
            Some(AuthConfig::Bearer { token }) => {
                assert_eq!(token, KEYRING_SECRET_PLACEHOLDER);
            }
            _ => panic!("expected bearer sentinel preserved in config.json"),
        }

        persist_user_config(&Config {
            base_url: "https://apikey.example".into(),
            download_dir: "/dl".into(),
            use_https: false,
            auth: Some(AuthConfig::ApiKey {
                header: "X-Api-Key".into(),
                key: KEYRING_SECRET_PLACEHOLDER.to_string(),
            }),
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig::default(),
            roms_layout: RomsLayoutConfig::default(),
            theme: default_theme_id(),
            tui_layout: TuiLayoutConfig::default(),
        })
        .expect("persist api key sentinel");

        let cfg: Config = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(cfg.base_url, "https://apikey.example");
        match cfg.auth {
            Some(AuthConfig::ApiKey { header, key }) => {
                assert_eq!(header, "X-Api-Key");
                assert_eq!(key, KEYRING_SECRET_PLACEHOLDER);
            }
            _ => panic!("expected api key sentinel preserved"),
        }

        persist_user_config(&Config {
            base_url: "https://basic.example".into(),
            download_dir: "/dl".into(),
            use_https: true,
            auth: Some(AuthConfig::Basic {
                username: "alice".into(),
                password: KEYRING_SECRET_PLACEHOLDER.to_string(),
            }),
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig::default(),
            roms_layout: RomsLayoutConfig::default(),
            theme: default_theme_id(),
            tui_layout: TuiLayoutConfig::default(),
        })
        .expect("persist basic password sentinel");

        let cfg: Config = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(cfg.base_url, "https://basic.example");
        match cfg.auth {
            Some(AuthConfig::Basic { username, password }) => {
                assert_eq!(username, "alice");
                assert_eq!(password, KEYRING_SECRET_PLACEHOLDER);
            }
            _ => panic!("expected basic password sentinel preserved"),
        }
    }

    #[test]
    fn should_check_updates_defaults_true_and_honors_false_values() {
        let _env = TestEnv::new();
        std::env::remove_var("ROMM_CHECK_UPDATES");
        assert!(should_check_updates());

        for value in ["false", "FALSE", "0", "no", "off"] {
            std::env::set_var("ROMM_CHECK_UPDATES", value);
            assert!(
                !should_check_updates(),
                "expected ROMM_CHECK_UPDATES={value} to disable checks"
            );
        }

        std::env::set_var("ROMM_CHECK_UPDATES", "true");
        assert!(should_check_updates());
    }
}
