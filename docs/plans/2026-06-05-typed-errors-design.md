# Typed Errors Design (Gap 1)

## Overview

Replace `anyhow::Result` at library boundaries with `thiserror` domain enums composed into `RommError`. Reserve `anyhow` for the binary entrypoint (`main.rs`) where typed errors are converted to user-facing messages and exit codes.

## Error types

### `ApiError` — HTTP client boundary

```rust
pub enum ApiError {
    Unauthorized { body: String },
    Forbidden { body: String },
    NotFound { path: String, body: String },
    RateLimited { retry_after: Option<u64>, body: String },
    ClientError { status: u16, body: String },
    ServerError { status: u16, body: String },
    Request(#[from] reqwest::Error),
    Decode(#[from] serde_json::Error),
    InvalidMethod(String),
    InvalidHeader(String),
}
```

- `from_http_response(status, body)` maps HTTP status codes to variants.
- `status_code()` returns `Some(u16)` for HTTP variants (tests, TUI).
- `is_auth_failure()` is true for `Unauthorized` and `Forbidden`.
- `Display` preserves legacy format: `ROMM API error: {status} {reason} - {body}`.

### `ConfigError` — configuration and keyring

```rust
pub enum ConfigError {
    MissingBaseUrl,
    TokenFileRead { path: String, #[source] source: std::io::Error },
    TokenFileTooLarge { max: usize },
    TokenFileInvalidUtf8 { path: String },
    TokenFileEmpty { path: String },
    KeyringEntry { key: String, message: String },
    KeyringStore { key: String, message: String },
    ConfigDirUnavailable,
    InvalidConfigPath,
    Io { context: String, #[source] source: std::io::Error },
}
```

### `DownloadError` — streaming downloads and paths

```rust
pub enum DownloadError {
    Io(#[from] std::io::Error),
    Cancelled(#[from] CancelledByUser),
    PathNotConfigured,
    RomsDirEmpty,
    InvalidRomsDir { path: String },
    Api(#[from] ApiError),
    JobListPoisoned(String),
    FailedWithoutDetails,
    RenameFailed { path: String, #[source] source: std::io::Error },
}
```

### `RommError` — public composed type

```rust
pub enum RommError {
    #[error(transparent)] Api(#[from] ApiError),
    #[error(transparent)] Config(#[from] ConfigError),
    #[error(transparent)] Download(#[from] DownloadError),
}
```

## Frontend mapping

| Variant | CLI hint | TUI hint | Exit code |
|---------|----------|----------|-----------|
| `Config(MissingBaseUrl)` | run `romm-cli init` | SetupWizard | 3 |
| `Config(*)` (other) | check config / keyring | Settings | 3 |
| `Api(Unauthorized \| Forbidden)` | run `romm-cli auth` | ReAuth | 3 |
| `Api(ServerError \| ClientError)` | server/API error | Retry | 4 |
| `Api(Request)` | network error | Retry | 4 |
| `Download(Cancelled)` | (quiet) | Dismiss | 0 |
| `Download(*)` | download failed | Retry | 4 |
| Other | generic failure | Dismiss | 1 |

Helpers: `user_message(&RommError)`, `exit_code(&RommError)`, `tui_hint(&RommError)`, `RommError::is_cancelled()`.

## Migration layers

1. **Foundation** — `src/error.rs`, `thiserror` dependency.
2. **Client** — `RommClient` methods return `ApiError` (or `Result<T, ApiError>`).
3. **Config** — `load_config`, `persist_user_config` return `ConfigError`.
4. **Download** — `core/download/*` returns `DownloadError`; `CancelledByUser` via `#[from]`.
5. **Frontends** — `main` maps `RommError` to exit codes; TUI `set_error(RommError)`.
6. **Tests/docs** — assert on `status_code()`, update guidelines.

## Rules

- No `anyhow` in public signatures of `RommClient`, `load_config`, or download APIs.
- Callers that still use `anyhow` convert with `.map_err(RommError::from)?` or `RommError::from(e)`.
- `main.rs` converts `RommError` → display string + `exit_code()` at the binary boundary.
