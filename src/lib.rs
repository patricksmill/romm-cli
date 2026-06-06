//! # romm-cli
//!
//! `romm-cli` is a powerful command-line interface and terminal user interface (TUI)
//! for interacting with the [RomM](https://github.com/romm-apps/romm) API.
//!
//! It provides tools for:
//! - Browsing and searching your ROM collection.
//! - Downloading ROMs and game saves.
//! - Uploading new ROMs and saves.
//! - Managing server-side tasks (library scans, etc.).
//! - Securely managing authentication via the OS keyring.
//!
//! ## Quick Start
//!
//! Most users will interact with the crate through the `romm-cli` binary.
//! For library consumers, the core entry point is the [`client::RommClient`].
//!
//! ```no_run
//! use romm_cli::config::load_config;
//! use romm_cli::client::RommClient;
//! use romm_cli::error::RommError;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), RommError> {
//!     let config = load_config()?;
//!     let client = RommClient::new(&config, false)?;
//!
//!     let version = client.rom_server_version_from_heartbeat().await;
//!     println!("Connected to RomM server version: {:?}", version);
//!
//!     Ok(())
//! }
//! ```

/// CLI output presentation (color, progress, JSON vs text).
pub mod cli_presentation;
/// HTTP client implementation for the RomM API.
pub mod client;
/// CLI command handlers.
pub mod commands;
/// Configuration and authentication management.
pub mod config;
/// Internal core logic and shared utilities.
pub mod core;
/// Type-safe API endpoint definitions.
pub mod endpoints;
/// Typed error hierarchy (`ApiError`, `ConfigError`, `DownloadError`, `RommError`).
pub mod error;
/// Redaction helpers for tracing output (no secrets in logs).
pub mod log_redact;
pub use error::exit;
/// Feature compatibility helpers based on OpenAPI endpoint availability.
pub mod feature_compat;
/// Frontend-specific logic (shared between CLI and TUI).
pub mod frontend;
/// OpenAPI parsing and endpoint lookup helpers.
pub mod openapi;
/// TUI implementation (requires the `tui` feature).
#[cfg(feature = "tui")]
pub mod tui;
/// Shared data models and types.
pub mod types;
/// Auto-update logic.
pub mod update;
