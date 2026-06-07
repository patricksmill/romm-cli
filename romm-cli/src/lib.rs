//! # romm-cli
//!
//! Command-line interface for the [RomM](https://github.com/romm-apps/romm) API.
//! Re-exports [`romm_api`] for library consumers that depend on the `romm-cli` crate.

/// CLI output presentation (color, progress, JSON vs text).
pub mod cli_presentation;
/// CLI command handlers.
pub mod commands;
/// CLI frontend routing.
pub mod frontend;

pub use romm_api::client;
pub use romm_api::config;
pub use romm_api::core;
pub use romm_api::endpoints;
pub use romm_api::error;
pub use romm_api::exit;
pub use romm_api::feature_compat;
pub use romm_api::log_redact;
pub use romm_api::openapi;
pub use romm_api::types;
pub mod update;
