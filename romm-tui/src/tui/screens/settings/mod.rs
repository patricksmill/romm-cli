//! Interactive settings screen for editing current config.

mod console;
mod render;
mod state;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    ConsolePathKind, SettingsConfirm, SettingsPickerKind, SettingsRow, SettingsScreen, SettingsTab,
};
