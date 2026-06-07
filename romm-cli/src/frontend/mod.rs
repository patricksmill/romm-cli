//! Frontend routing helpers.

pub mod cli;

#[cfg(feature = "tui")]
pub use romm_tui::frontend::tui;
