//! Frontend routing helpers.
//!
//! This module keeps runtime selection between presentation layers
//! (CLI and TUI) separate from core business logic.

pub mod cli;
#[cfg(feature = "tui")]
pub mod tui;
