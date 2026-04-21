//! Frontend-agnostic core modules.
//!
//! This module groups reusable state and utilities that should be shared
//! across CLI, TUI, and future GUI frontends.

pub mod cache;
pub mod download;
pub mod interrupt;
pub mod startup_library_snapshot;
pub mod utils;
