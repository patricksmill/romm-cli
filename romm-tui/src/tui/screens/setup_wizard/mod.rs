//! First-run setup: server URL, ROMs directory, authentication, test connection, persist config.

mod config;
pub(crate) mod event;
mod input;
mod layout;
mod render;
mod run;
mod types;

#[cfg(test)]
mod tests;

pub use types::SetupWizard;

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}
