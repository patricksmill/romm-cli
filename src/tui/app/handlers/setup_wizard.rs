//! Setup wizard key handler.

use crate::client::RommClient;
use crate::tui::theme::MessageTone;
use anyhow::Result;
use crossterm::event::KeyEvent;

use super::super::{App, AppScreen};
use crate::tui::screens::SettingsScreen;

impl App {
    pub(in crate::tui::app) async fn handle_setup_wizard(
        &mut self,
        key: &KeyEvent,
    ) -> Result<bool> {
        let wizard = match &mut self.screen {
            AppScreen::SetupWizard(w) => w,
            _ => return Ok(false),
        };

        use crate::tui::screens::setup_wizard::event::SetupAction;
        if wizard.update(SetupAction::Key(*key))? {
            // Esc pressed
            self.apply_saved_theme();
            self.screen = AppScreen::Settings(Box::new(SettingsScreen::new(
                &self.config,
                self.server_version.as_deref(),
                self.save_sync_compat.clone(),
            )));
            return Ok(false);
        }

        if wizard.testing {
            let result = wizard.try_connect_and_persist(self.client.verbose()).await;
            wizard.testing = false;
            match result {
                Ok(cfg) => {
                    let auth_ok = cfg.auth.is_some();
                    self.config = cfg;
                    if let Ok(new_client) = RommClient::new(&self.config, self.client.verbose()) {
                        self.client = new_client;
                    }
                    let mut settings = SettingsScreen::new(
                        &self.config,
                        self.server_version.as_deref(),
                        self.save_sync_compat.clone(),
                    );
                    if auth_ok {
                        settings.message = Some((
                            "Authentication updated successfully".to_string(),
                            MessageTone::Success,
                        ));
                    } else {
                        settings.message = Some((
                            "Saved configuration but credentials could not be loaded from the OS keyring (see logs)."
                                .to_string(),
                            MessageTone::Warning,
                        ));
                    }
                    self.apply_saved_theme();
                    self.screen = AppScreen::Settings(Box::new(settings));
                }
                Err(e) => {
                    wizard.error = Some(format!("{e:#}"));
                }
            }
        }
        Ok(false)
    }
}
