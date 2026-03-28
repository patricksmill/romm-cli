//! Library root for `romm-cli`: HTTP client, CLI/TUI frontends, and shared core.

pub mod client;
pub mod commands;
pub mod config;
pub mod core;
pub mod endpoints;
pub mod frontend;
pub mod services;
#[cfg(feature = "tui")]
pub mod tui;
pub mod types;
