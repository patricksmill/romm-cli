//! Standalone first-run setup event loop.

use anyhow::{anyhow, Result};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;

use crate::config::{default_theme_id, Config};
use crate::tui::theme::{resolve_theme_or_default, RommStyles};

use super::types::SetupWizard;

impl SetupWizard {
    pub async fn run(mut self, verbose: bool) -> Result<Config> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            crossterm::event::EnableBracketedPaste
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let theme = resolve_theme_or_default(&default_theme_id());

        loop {
            let styles = RommStyles::new(theme.as_ref());
            terminal.draw(|f| {
                let area = f.area();
                self.render(f, area, &styles);
                if let Some((x, y)) = self.cursor_pos(area) {
                    f.set_cursor_position((x, y));
                }
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                let ev = event::read()?;
                let mut should_exit = false;

                match ev {
                    Event::Key(key) if self.handle_key(&key)? => {
                        should_exit = true;
                    }
                    Event::Paste(text) => {
                        self.handle_paste(&text);
                    }
                    _ => {}
                }

                if should_exit {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        crossterm::event::DisableBracketedPaste,
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    return Err(anyhow!("setup cancelled"));
                }

                if self.testing {
                    let styles = RommStyles::new(theme.as_ref());
                    terminal.draw(|f| {
                        let area = f.area();
                        self.render(f, area, &styles);
                    })?;
                    let result = self.try_connect_and_persist(verbose).await;
                    self.testing = false;
                    match result {
                        Ok(cfg) => {
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                crossterm::event::DisableBracketedPaste,
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;
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
