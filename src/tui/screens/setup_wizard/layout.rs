//! Layout helpers for the setup wizard screen.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Text};

use crate::config::{read_user_config_json_from_disk, ExtrasDefaults};
use crate::tui::theme::RommStyles;

use super::types::Step;

pub(crate) fn extras_defaults_from_disk() -> ExtrasDefaults {
    read_user_config_json_from_disk()
        .map(|c| c.extras_defaults)
        .unwrap_or_default()
}

pub(crate) fn wizard_layout(area: Rect, step: Step) -> [Rect; 3] {
    let top = if matches!(step, Step::Url) { 5 } else { 3 };
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top),
            Constraint::Min(6),
            Constraint::Length(4),
        ])
        .split(area);
    [v[0], v[1], v[2]]
}

pub(crate) fn wizard_footer_text<'a>(keys: &'a str, styles: &RommStyles<'_>) -> Text<'a> {
    let ver = format!("romm-cli {}", env!("CARGO_PKG_VERSION"));
    Text::from(vec![
        Line::from(keys).style(styles.label()),
        Line::from(ver).style(styles.muted()),
    ])
}
