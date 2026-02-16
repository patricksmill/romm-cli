//! Frontend routing helpers.
//!
//! This module keeps runtime selection between presentation layers
//! (CLI, TUI, and future GUI) separate from core business logic.

pub mod cli;
pub mod gui_slint;
pub mod tui;
