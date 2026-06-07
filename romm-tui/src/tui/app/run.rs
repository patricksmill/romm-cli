//! Main TUI event loop.

use anyhow::Result;

use crate::tui::runtime::{LoopControl, RuntimeOptions, TuiSession};

use super::event::{Action, AppEvent};
use super::App;

impl App {
    pub async fn run(&mut self) -> Result<()> {
        self.open_library_browse();
        let mut session = TuiSession::enter(RuntimeOptions::default())?;
        let mut quit = false;

        while !quit {
            let events = self.poll_frame_events()?;
            for event in events {
                for action in self.map_event(event) {
                    if self.update(action).await? {
                        quit = true;
                        break;
                    }
                }
                if quit {
                    break;
                }
            }

            session.terminal_mut().draw(|f| self.render(f))?;

            if self.on_frame_end().await? == LoopControl::Break {
                quit = true;
            }
        }

        session.leave()?;
        Ok(())
    }

    fn poll_frame_events(&mut self) -> Result<Vec<AppEvent>> {
        let mut events = self.drain_background_events();
        if self
            .startup_splash
            .as_ref()
            .is_some_and(|s| s.should_auto_dismiss())
        {
            events.push(AppEvent::AutoDismissSplash);
        }

        if self
            .startup_update_prompt
            .as_ref()
            .is_some_and(|p| p.updating)
        {
            return Ok(events);
        }

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => events.push(AppEvent::Key(key)),
                crossterm::event::Event::Paste(text) => events.push(AppEvent::Paste(text)),
                _ => {}
            }
        }
        Ok(events)
    }

    async fn on_frame_end(&mut self) -> Result<LoopControl> {
        if self
            .startup_update_prompt
            .as_ref()
            .is_some_and(|p| p.updating)
        {
            self.update(Action::ApplyStartupUpdate).await?;
            return Ok(LoopControl::Continue);
        }
        self.update(Action::ProcessDeferredRomLoad).await?;
        Ok(LoopControl::Continue)
    }
}
