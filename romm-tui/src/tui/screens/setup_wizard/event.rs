//! Setup wizard event → action mapping.

use crossterm::event::KeyEvent;

/// Raw input for the setup wizard loop.
#[derive(Debug)]
pub enum SetupEvent {
    Key(KeyEvent),
    Paste(String),
}

/// Semantic intents for [`super::SetupWizard::update`].
#[derive(Debug)]
pub enum SetupAction {
    Key(KeyEvent),
    Paste(String),
}

pub fn map_setup_event(event: SetupEvent) -> SetupAction {
    match event {
        SetupEvent::Key(key) => SetupAction::Key(key),
        SetupEvent::Paste(text) => SetupAction::Paste(text),
    }
}

impl super::SetupWizard {
    /// Returns `true` when the wizard should exit (cancelled).
    pub fn update(&mut self, action: SetupAction) -> anyhow::Result<bool> {
        match action {
            SetupAction::Key(key) => self.handle_key(&key),
            SetupAction::Paste(text) => {
                self.handle_paste(&text);
                Ok(false)
            }
        }
    }
}
