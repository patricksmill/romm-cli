//! Full-screen “connected” banner after setup or on successful server contact.

use std::time::{Duration, Instant};

use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::RommStyles;

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

pub fn render(f: &mut Frame, area: Rect, splash: &StartupSplash, styles: &RommStyles) {
    let ver_line = splash
        .server_version
        .as_ref()
        .map(|v| format!("RomM server version: {v}"))
        .unwrap_or_else(|| "Connected (heartbeat version unavailable)".to_string());

    let lines = vec![
        Line::from(vec![Span::styled(
            "✓ Connected",
            styles.success().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            splash.base_url.to_string(),
            styles.primary_text(),
        )),
        Line::from(Span::styled(ver_line, styles.muted())),
        Line::from(""),
        Line::from(Span::styled(
            "Enter or Esc — continue",
            styles.footer_hint(),
        )),
    ];

    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            styles
                .panel_block("romm-cli")
                .border_style(styles.success()),
        );
    f.render_widget(p, area);
}
