//! Standalone first-run setup event loop.

use anyhow::{anyhow, Result};
use crossterm::event::{self, Event};

use crate::tui::runtime::{RuntimeOptions, TuiSession};
use crate::tui::theme::{resolve_theme_or_default, RommStyles};
use romm_api::config::{default_theme_id, Config};

use super::event::{map_setup_event, SetupEvent};
use super::types::SetupWizard;

impl SetupWizard {
    pub async fn run(mut self, verbose: bool) -> Result<Config> {
        let mut session = TuiSession::enter(RuntimeOptions {
            bracketed_paste: true,
        })?;
        let theme = resolve_theme_or_default(&default_theme_id());

        loop {
            let styles = RommStyles::new(theme.as_ref());
            session.terminal_mut().draw(|f| {
                let area = f.area();
                self.render(f, area, &styles);
                if let Some((x, y)) = self.cursor_pos(area) {
                    f.set_cursor_position((x, y));
                }
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                let setup_event = match event::read()? {
                    Event::Key(key) => SetupEvent::Key(key),
                    Event::Paste(text) => SetupEvent::Paste(text),
                    _ => continue,
                };
                let action = map_setup_event(setup_event);
                if self.update(action)? {
                    session.leave()?;
                    return Err(anyhow!("setup cancelled"));
                }

                if self.testing {
                    let styles = RommStyles::new(theme.as_ref());
                    session.terminal_mut().draw(|f| {
                        let area = f.area();
                        self.render(f, area, &styles);
                    })?;
                    let result = self.try_connect_and_persist(verbose).await;
                    self.testing = false;
                    match result {
                        Ok(cfg) => {
                            session.leave()?;
                            return Ok(cfg);
                        }
                        Err(e) => {
                            self.error = Some(format!("{e:#}"));
                        }
                    }
                }
            }
        }
    }
}
