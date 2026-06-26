//! Typed error hierarchy for the romm-cli library.
//!
//! Domain enums use `thiserror`; the binary boundary converts [`RommError`] to
//! user-facing messages and exit codes via [`user_message`], [`exit_code`], and [`exit`].

use reqwest::StatusCode;
use thiserror::Error;

use crate::core::interrupt::CancelledByUser;

// ---------------------------------------------------------------------------
// ApiError
// ---------------------------------------------------------------------------

/// HTTP and API-layer failures from [`crate::client::RommClient`].
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("ROMM API error: 401 Unauthorized - {body}")]
    Unauthorized { body: String },

    #[error("ROMM API error: 403 Forbidden - {body}")]
    Forbidden { body: String },

    #[error("ROMM API error: 404 Not Found - {body}")]
    NotFound { path: String, body: String },

    #[error("ROMM API error: 429 Too Many Requests - {body}")]
    RateLimited {
        retry_after: Option<u64>,
        body: String,
    },

    #[error("ROMM API error: {status} - {body}")]
    ClientError { status: u16, body: String },

    #[error("ROMM API error: {status} - {body}")]
    ServerError { status: u16, body: String },

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("invalid response: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("invalid HTTP header: {0}")]
    InvalidHeader(String),

    #[error("unexpected API response: {0}")]
    UnexpectedResponse(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl ApiError {
    /// Map an HTTP status code and response body to a typed variant.
    pub fn from_http_response(status: StatusCode, body: impl Into<String>) -> Self {
        let body = body.into();
        match status.as_u16() {
            401 => Self::Unauthorized { body },
            403 => Self::Forbidden { body },
            404 => Self::NotFound {
                path: String::new(),
                body,
            },
            429 => Self::RateLimited {
                retry_after: None,
                body,
            },
            500..=599 => Self::ServerError {
                status: status.as_u16(),
                body,
            },
            400..=499 => Self::ClientError {
                status: status.as_u16(),
                body,
            },
            _ => Self::ClientError {
                status: status.as_u16(),
                body,
            },
        }
    }

    /// HTTP status when this error represents an API response failure.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Unauthorized { .. } => Some(401),
            Self::Forbidden { .. } => Some(403),
            Self::NotFound { .. } => Some(404),
            Self::RateLimited { .. } => Some(429),
            Self::ClientError { status, .. } | Self::ServerError { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// True for 401/403 — callers may prompt re-authentication.
    pub fn is_auth_failure(&self) -> bool {
        matches!(self, Self::Unauthorized { .. } | Self::Forbidden { .. })
    }

    /// True when the error body or display text indicates a 404 (URL fallback logic).
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
            || self.status_code().is_some_and(|s| s == 404)
            || self.to_string().contains("404 Not Found")
    }

    /// HTTP response body when the server returned one (for UI detail lines).
    pub fn response_body(&self) -> Option<&str> {
        match self {
            Self::Unauthorized { body }
            | Self::Forbidden { body }
            | Self::NotFound { body, .. }
            | Self::RateLimited { body, .. }
            | Self::ClientError { body, .. }
            | Self::ServerError { body, .. } => Some(body.as_str()),
            _ => None,
        }
    }

    /// Log-safe summary: status and variant only; HTTP response bodies are omitted.
    pub fn redacted_for_log(&self) -> String {
        match self {
            Self::Unauthorized { .. } => "ApiError: 401 Unauthorized (body redacted)".to_string(),
            Self::Forbidden { .. } => "ApiError: 403 Forbidden (body redacted)".to_string(),
            Self::NotFound { path, .. } => {
                format!("ApiError: 404 Not Found path={path} (body redacted)")
            }
            Self::RateLimited { retry_after, .. } => format!(
                "ApiError: 429 Too Many Requests retry_after={retry_after:?} (body redacted)"
            ),
            Self::ClientError { status, .. } => {
                format!("ApiError: client error {status} (body redacted)")
            }
            Self::ServerError { status, .. } => {
                format!("ApiError: server error {status} (body redacted)")
            }
            Self::Request(e) => format!("ApiError: request failed: {e}"),
            Self::Decode(e) => format!("ApiError: invalid response: {e}"),
            Self::InvalidMethod(m) => format!("ApiError: invalid HTTP method: {m}"),
            Self::InvalidHeader(h) => format!("ApiError: invalid HTTP header: {h}"),
            Self::UnexpectedResponse(m) => format!("ApiError: unexpected API response: {m}"),
            Self::Io(e) => format!("ApiError: I/O error: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// ConfigError
// ---------------------------------------------------------------------------

/// Configuration, token file, and keyring failures.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "API_BASE_URL is not set. Set it in the environment, a config.json file, or run: romm-cli init"
    )]
    MissingBaseUrl,

    #[error("read bearer token file {path}: {source}")]
    TokenFileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("bearer token file exceeds max size of {max} bytes")]
    TokenFileTooLarge { max: usize },

    #[error("bearer token file must be valid UTF-8: {path}")]
    TokenFileInvalidUtf8 { path: String },

    #[error("bearer token file is empty after trimming whitespace: {path}")]
    TokenFileEmpty { path: String },

    #[error("keyring entry error for {key}: {message}")]
    KeyringEntry { key: String, message: String },

    #[error("keyring store error for {key}: {message}")]
    KeyringStore { key: String, message: String },

    #[error("Could not resolve config directory. Set ROMM_OPENAPI_PATH to store openapi.json.")]
    ConfigDirUnavailable,

    #[error("Could not determine config directory (no HOME / APPDATA?).")]
    ConfigDirNotFound,

    #[error("invalid config path")]
    InvalidConfigPath,

    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("{0}")]
    Other(String),

    #[error("failed to serialize config: {0}")]
    Serialize(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// DownloadError
// ---------------------------------------------------------------------------

/// Download, path resolution, and transfer failures.
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("operation cancelled by user")]
    Cancelled(#[from] CancelledByUser),

    #[error("ROMs directory is not configured. Run setup to set a ROMs path.")]
    PathNotConfigured,

    #[error("ROMs directory cannot be empty")]
    RomsDirEmpty,

    #[error("ROMs directory is not valid: {path}")]
    InvalidRomsDir { path: String },

    #[error(transparent)]
    Api(#[from] ApiError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("download job list lock poisoned: {0}")]
    JobListPoisoned(String),

    #[error("download failed without error details")]
    FailedWithoutDetails,

    #[error("Could not move temp ROM {path} to final destination {final_path}: {source}")]
    RenameFailed {
        path: String,
        final_path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("no extras targets selected")]
    NoExtrasTargets,

    #[error("extras job list lock poisoned: {0}")]
    ExtrasJobListPoisoned(String),

    #[error("{context}: {source}")]
    IoContext {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("{0}")]
    Unexpected(String),
}

impl DownloadError {
    /// True when the underlying API error is a 404 (URL fallback logic).
    pub fn is_not_found(&self) -> bool {
        matches!(self, DownloadError::Api(api) if api.is_not_found())
    }
}

// ---------------------------------------------------------------------------
// RommError
// ---------------------------------------------------------------------------

/// Composed public error type for library consumers.
#[derive(Debug, Error)]
pub enum RommError {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Download(#[from] DownloadError),

    #[error("{0}")]
    Other(String),
}

/// Converts a binary-boundary `anyhow::Error` into [`RommError`], preserving typed
/// domain errors that were bubbled via `?` from legacy command handlers.
pub fn from_anyhow(err: anyhow::Error) -> RommError {
    match err.downcast::<ApiError>() {
        Ok(api) => RommError::Api(api),
        Err(err) => match err.downcast::<ConfigError>() {
            Ok(cfg) => RommError::Config(cfg),
            Err(err) => match err.downcast::<DownloadError>() {
                Ok(dl) => RommError::Download(dl),
                Err(err) => match err.downcast::<RommError>() {
                    Ok(re) => re,
                    Err(err) => RommError::Other(err.to_string()),
                },
            },
        },
    }
}

impl RommError {
    /// True when the user cancelled a long-running operation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, RommError::Download(DownloadError::Cancelled(_)))
    }

    /// True for auth-related API or config failures.
    pub fn is_auth_or_config(&self) -> bool {
        match self {
            RommError::Config(_) => true,
            RommError::Api(api) => api.is_auth_failure(),
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Frontend mapping
// ---------------------------------------------------------------------------

/// Hint for TUI error toast behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiErrorHint {
    /// Prompt user to run setup / init.
    RunInit,
    /// Prompt user to re-authenticate in Settings.
    ReAuth,
    /// Transient failure — user may retry.
    Retry,
    /// Generic dismissible error.
    Dismiss,
}

/// Actionable user-facing message (short, no full error chain).
pub fn user_message(err: &RommError) -> String {
    if err.is_cancelled() {
        return "Operation cancelled.".to_string();
    }
    match err {
        RommError::Config(ConfigError::MissingBaseUrl) => {
            "API_BASE_URL is not set. Run `romm-cli init` to configure.".to_string()
        }
        RommError::Config(_) => {
            format!("Configuration error: {err}. Check config or run `romm-cli init`.")
        }
        RommError::Api(api) if api.is_auth_failure() => {
            "Authentication failed. Check credentials or run `romm-cli auth`.".to_string()
        }
        RommError::Api(ApiError::Forbidden { .. }) => {
            "Access denied. Check credentials or run `romm-cli auth`.".to_string()
        }
        RommError::Api(ApiError::NotFound { .. }) => {
            "Resource not found. Check the server URL and resource ID.".to_string()
        }
        RommError::Api(ApiError::RateLimited { .. }) => {
            "Rate limited by the server. Wait a moment and try again.".to_string()
        }
        RommError::Api(ApiError::ClientError { status, .. }) if (400..500).contains(status) => {
            format!("Request rejected ({status}). Check command arguments and try again.")
        }
        RommError::Api(ApiError::Request(_)) => {
            "Network error. Check your connection and server URL.".to_string()
        }
        RommError::Api(ApiError::ServerError { .. }) => {
            "Server error. Try again later.".to_string()
        }
        RommError::Download(DownloadError::PathNotConfigured) => {
            "ROMs directory is not configured. Run `romm-cli init`.".to_string()
        }
        RommError::Download(DownloadError::IoContext { .. }) => {
            format!("Download I/O error: {err}. Check disk permissions and output path.")
        }
        RommError::Download(_) => format!("Download failed: {err}"),
        RommError::Api(_) => format!("API error: {err}"),
        RommError::Other(msg) => msg.clone(),
    }
}

/// Like [`user_message`], but appends a truncated HTTP response body when present.
pub fn user_message_with_server_detail(err: &RommError, max_body_chars: usize) -> String {
    let base = user_message(err);
    let RommError::Api(api) = err else {
        return base;
    };
    let Some(body) = api.response_body().map(str::trim).filter(|b| !b.is_empty()) else {
        return base;
    };
    let detail = crate::core::utils::truncate(body, max_body_chars);
    if let Some(status) = api.status_code() {
        format!("{base} (HTTP {status}: {detail})")
    } else {
        format!("{base} ({detail})")
    }
}

/// Process exit codes for scripting (see README "Exit codes").
pub mod exit {
    /// Command completed successfully (including user-cancelled downloads).
    pub const SUCCESS: i32 = 0;
    /// Unexpected or app-level validation failure.
    pub const GENERAL: i32 = 1;
    /// Invalid flags or arguments (clap parse errors only).
    pub const USAGE: i32 = 2;
    /// Configuration or authentication failure.
    pub const CONFIG: i32 = 3;
    /// API, network, or download failure.
    pub const API: i32 = 4;
}

/// Maps a [`RommError`] to a process exit code for the `romm-cli` binary.
pub fn exit_code(err: &RommError) -> i32 {
    if err.is_cancelled() {
        return exit::SUCCESS;
    }
    match err {
        RommError::Config(_) => exit::CONFIG,
        RommError::Api(api) if api.is_auth_failure() => exit::CONFIG,
        RommError::Api(ApiError::Request(_)) => exit::API,
        RommError::Api(_) => exit::API,
        RommError::Download(_) => exit::API,
        RommError::Other(_) => exit::GENERAL,
    }
}

/// TUI behavior hint for an error.
pub fn tui_hint(err: &RommError) -> TuiErrorHint {
    if err.is_cancelled() {
        return TuiErrorHint::Dismiss;
    }
    match err {
        RommError::Config(ConfigError::MissingBaseUrl) => TuiErrorHint::RunInit,
        RommError::Config(_) => TuiErrorHint::ReAuth,
        RommError::Api(api) if api.is_auth_failure() => TuiErrorHint::ReAuth,
        RommError::Api(ApiError::Request(_))
        | RommError::Api(ApiError::ServerError { .. })
        | RommError::Download(_) => TuiErrorHint::Retry,
        RommError::Other(_) | RommError::Api(_) => TuiErrorHint::Dismiss,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_http_response_maps_status_codes() {
        let e = ApiError::from_http_response(StatusCode::UNAUTHORIZED, "bad token");
        assert!(matches!(e, ApiError::Unauthorized { .. }));
        assert_eq!(e.status_code(), Some(401));
        assert!(e.is_auth_failure());

        let e = ApiError::from_http_response(StatusCode::INTERNAL_SERVER_ERROR, "oops");
        assert!(matches!(e, ApiError::ServerError { status: 500, .. }));
        assert_eq!(e.status_code(), Some(500));

        let e = ApiError::from_http_response(StatusCode::NOT_FOUND, "missing");
        assert!(matches!(e, ApiError::NotFound { .. }));
        assert!(e.is_not_found());
    }

    #[test]
    fn romm_error_is_cancelled() {
        let err = RommError::Download(DownloadError::Cancelled(CancelledByUser));
        assert!(err.is_cancelled());
        assert_eq!(exit_code(&err), 0);
    }

    #[test]
    fn exit_code_auth_vs_network() {
        let auth = RommError::Api(ApiError::Unauthorized { body: "x".into() });
        assert_eq!(exit_code(&auth), exit::CONFIG);

        let net = RommError::Api(ApiError::ServerError {
            status: 503,
            body: "down".into(),
        });
        assert_eq!(exit_code(&net), exit::API);
    }

    #[test]
    fn exit_code_maps_all_variants() {
        assert_eq!(
            exit_code(&RommError::Config(ConfigError::MissingBaseUrl)),
            exit::CONFIG
        );

        let forbidden = RommError::Api(ApiError::Forbidden {
            body: "denied".into(),
        });
        assert_eq!(exit_code(&forbidden), exit::CONFIG);

        // `Request` branch is covered by CLI integration tests; `ClientError` shares the API arm.
        let api = RommError::Api(ApiError::ClientError {
            status: 502,
            body: "bad gateway".into(),
        });
        assert_eq!(exit_code(&api), exit::API);

        assert_eq!(
            exit_code(&RommError::Download(DownloadError::PathNotConfigured)),
            exit::API
        );

        assert_eq!(exit_code(&RommError::Other("x".into())), exit::GENERAL);
    }

    #[test]
    fn user_message_actionable_hints() {
        let not_found = RommError::Api(ApiError::NotFound {
            path: "/api/x".into(),
            body: "missing".into(),
        });
        assert!(user_message(&not_found).contains("server URL"));

        let forbidden = RommError::Api(ApiError::Forbidden {
            body: "denied".into(),
        });
        assert!(user_message(&forbidden).contains("romm-cli auth"));

        let rate = RommError::Api(ApiError::RateLimited {
            retry_after: Some(30),
            body: "slow down".into(),
        });
        assert!(user_message(&rate).contains("Rate limited"));
    }

    #[test]
    fn user_message_with_server_detail_includes_body() {
        let err = RommError::Api(ApiError::ServerError {
            status: 500,
            body: "No metadata providers enabled".into(),
        });
        let msg = user_message_with_server_detail(&err, 120);
        assert!(msg.contains("Server error"));
        assert!(msg.contains("No metadata providers enabled"));
        assert!(msg.contains("HTTP 500"));
    }
}
