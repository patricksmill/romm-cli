//! Full-screen “connected” banner after setup or on successful server contact.

use std::time::{Duration, Instant};

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Shown briefly before the main menu when the server is reachable.
pub struct StartupSplash {
    pub base_url: String,
    pub server_version: Option<String>,
    started: Instant,
}

impl StartupSplash {
    pub fn new(base_url: String, server_version: Option<String>) -> Self {
        Self {
            base_url,
            server_version,
            started: Instant::now(),
        }
    }

    pub fn should_auto_dismiss(&self) -> bool {
        self.started.elapsed() > Duration::from_millis(2800)
    }
}

pub fn render(f: &mut Frame, area: Rect, splash: &StartupSplash) {
    let ver_line = splash
        .server_version
        .as_ref()
        .map(|v| format!("RomM server version: {v}"))
        .unwrap_or_else(|| "Connected (heartbeat version unavailable)".to_string());

    let lines = vec![
        Line::from(vec![Span::styled(
            "✓ Connected",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            splash.base_url.to_string(),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(ver_line, Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled(
            "Enter or Esc — continue",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let p = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .title("romm-cli")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    f.render_widget(p, area);
}
