//! # romm-api
//!
//! Shared HTTP client, API types, and domain logic for RomM frontends (CLI, TUI, romm-rust-android).
//!
//! ## Quick Start
//!
//! ```no_run
//! use romm_api::config::load_config;
//! use romm_api::client::RommClient;
//! use romm_api::error::RommError;
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

/// HTTP client implementation for the RomM API.
pub mod client;
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
/// OpenAPI parsing and endpoint lookup helpers.
pub mod openapi;
/// Shared data models and types.
pub mod types;
/// Auto-update logic.
pub mod update;
