//! Shared terminal session setup for TUI frontends.

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

/// Options for terminal setup/teardown.
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeOptions {
    pub bracketed_paste: bool,
}

/// Control flow for the shared TUI loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopControl {
    Continue,
    Break,
}

/// Owns raw-mode terminal state until [`TuiSession::leave`].
pub struct TuiSession {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    options: RuntimeOptions,
}

impl TuiSession {
    pub fn enter(options: RuntimeOptions) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        if options.bracketed_paste {
            execute!(
                stdout,
                EnterAlternateScreen,
                EnableMouseCapture,
                crossterm::event::EnableBracketedPaste
            )?;
        } else {
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        }
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal, options })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<std::io::Stdout>> {
        &mut self.terminal
    }

    pub fn leave(mut self) -> Result<()> {
        disable_raw_mode()?;
        if self.options.bracketed_paste {
            execute!(
                self.terminal.backend_mut(),
                crossterm::event::DisableBracketedPaste,
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
        } else {
            execute!(
                self.terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
        }
        self.terminal.show_cursor()?;
        Ok(())
    }
}
