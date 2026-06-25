//! Layout helpers for the setup wizard screen.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Text};

use crate::tui::footer_hint::{footer_hint_line, FooterHintEntry};
use crate::tui::theme::RommStyles;
use romm_api::config::{read_user_config_json_from_disk, ExtrasDefaults};

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

const WIZARD_URL_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Backspace",
        label: "Delete",
    },
    FooterHintEntry {
        key: "Enter",
        label: "Next",
    },
];

const WIZARD_HTTPS_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Space",
        label: "Toggle",
    },
    FooterHintEntry {
        key: "Enter",
        label: "Next",
    },
];

const WIZARD_DOWNLOAD_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Ctrl+Enter",
        label: "Next",
    },
    FooterHintEntry {
        key: "↑",
        label: "Path bar",
    },
    FooterHintEntry {
        key: "Tab",
        label: "Path/list",
    },
];

const WIZARD_NEXT_HINTS: &[FooterHintEntry] = &[FooterHintEntry {
    key: "Enter",
    label: "Next",
}];

const WIZARD_AUTH_FIELD_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Tab",
        label: "Switch field",
    },
    FooterHintEntry {
        key: "Enter",
        label: "Next",
    },
];

const WIZARD_NEXT_STEP_HINTS: &[FooterHintEntry] = &[FooterHintEntry {
    key: "Enter",
    label: "Next step",
}];

const WIZARD_SUMMARY_HINTS: &[FooterHintEntry] = &[FooterHintEntry {
    key: "Enter",
    label: "Connect & save",
}];

pub(crate) fn wizard_footer_entries(step: Step) -> &'static [FooterHintEntry] {
    match step {
        Step::Url => WIZARD_URL_HINTS,
        Step::Https => WIZARD_HTTPS_HINTS,
        Step::Download => WIZARD_DOWNLOAD_HINTS,
        Step::CustomConsolePaths | Step::AuthMenu => WIZARD_NEXT_HINTS,
        Step::BasicUser | Step::BasicPass => WIZARD_AUTH_FIELD_HINTS,
        Step::Bearer | Step::PairingCode => WIZARD_NEXT_STEP_HINTS,
        Step::ApiHeader | Step::ApiKey => WIZARD_AUTH_FIELD_HINTS,
        Step::Summary => WIZARD_SUMMARY_HINTS,
    }
}

pub(crate) fn wizard_footer_text(
    entries: &[FooterHintEntry],
    inner_width: u16,
    styles: &RommStyles<'_>,
) -> Text<'static> {
    let ver = format!("romm-cli {}", env!("CARGO_PKG_VERSION"));
    Text::from(vec![
        footer_hint_line(entries, inner_width, styles),
        Line::from(ver).style(styles.muted()),
    ])
}
