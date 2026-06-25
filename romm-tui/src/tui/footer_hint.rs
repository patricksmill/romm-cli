//! Width-aware contextual footer hints with a permanent help anchor.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::tui::theme::RommStyles;

const SEPARATOR: &str = " | ";
const SEPARATOR_WIDTH: usize = 3;
const ELLIPSIS: &str = "…";
const ELLIPSIS_WIDTH: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FooterHintEntry {
    pub key: &'static str,
    pub label: &'static str,
}

pub const FOOTER_HELP_ANCHOR: FooterHintEntry = FooterHintEntry {
    key: "? / F1",
    label: "Help",
};

pub const PATH_PICKER_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Ctrl+Enter",
        label: "Apply path",
    },
    FooterHintEntry {
        key: "Tab",
        label: "Path/list",
    },
    FooterHintEntry {
        key: "Esc",
        label: "Cancel",
    },
];

pub fn entry_display_width(entry: &FooterHintEntry) -> usize {
    entry.key.width() + 2 + entry.label.width()
}

fn line_width(entries: &[FooterHintEntry], truncated: bool) -> usize {
    let mut width = 0;
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            width += SEPARATOR_WIDTH;
        }
        width += entry_display_width(entry);
    }
    if truncated {
        if !entries.is_empty() {
            width += SEPARATOR_WIDTH;
        }
        width += ELLIPSIS_WIDTH + SEPARATOR_WIDTH;
    } else if !entries.is_empty() {
        width += SEPARATOR_WIDTH;
    }
    width += entry_display_width(&FOOTER_HELP_ANCHOR);
    width
}

/// Fit priority-ordered entries (excluding anchor) into `width` display columns.
pub fn fit_footer_entries(entries: &[FooterHintEntry], width: u16) -> (Vec<FooterHintEntry>, bool) {
    let width = width as usize;
    if width == 0 {
        return (Vec::new(), false);
    }

    let mut best = 0usize;
    for count in 0..=entries.len() {
        let truncated = count < entries.len();
        if line_width(&entries[..count], truncated) <= width {
            best = count;
        }
    }

    let fitted: Vec<FooterHintEntry> = entries[..best].to_vec();
    let mut truncated = best < entries.len();
    if truncated && line_width(&fitted, true) > width && line_width(&[], false) <= width {
        truncated = false;
    }
    (fitted, truncated)
}

fn entry_spans(entry: &FooterHintEntry, styles: &RommStyles<'_>) -> Vec<Span<'static>> {
    vec![
        Span::styled(entry.key.to_string(), styles.label()),
        Span::styled(": ".to_string(), styles.footer_hint()),
        Span::styled(entry.label.to_string(), styles.footer_hint()),
    ]
}

/// Build a styled hint line that always ends with the help anchor.
pub fn footer_hint_line(
    entries: &[FooterHintEntry],
    width: u16,
    styles: &RommStyles<'_>,
) -> Line<'static> {
    let (fitted, truncated) = fit_footer_entries(entries, width);
    let mut spans = Vec::new();

    for (index, entry) in fitted.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(SEPARATOR));
        }
        spans.extend(entry_spans(entry, styles));
    }

    if truncated {
        if !fitted.is_empty() {
            spans.push(Span::raw(SEPARATOR));
        }
        spans.push(Span::styled(ELLIPSIS.to_string(), styles.footer_hint()));
        spans.push(Span::raw(SEPARATOR));
    } else if !fitted.is_empty() {
        spans.push(Span::raw(SEPARATOR));
    }

    spans.extend(entry_spans(&FOOTER_HELP_ANCHOR, styles));
    Line::from(spans)
}

pub fn footer_inner_width(area: Rect, styles: &RommStyles<'_>) -> u16 {
    styles.panel_block_untitled().inner(area).width
}

/// Render a bordered footer panel with optional metadata above the hint line.
pub fn render_footer_panel(
    f: &mut Frame,
    area: Rect,
    styles: &RommStyles<'_>,
    entries: &[FooterHintEntry],
    prefix_line: Option<&str>,
) {
    let block = styles.panel_block_untitled();
    let inner = block.inner(area);
    let hint_line = footer_hint_line(entries, inner.width, styles);

    let text = if let Some(prefix) = prefix_line.filter(|line| !line.is_empty()) {
        Text::from(vec![
            Line::from(Span::styled(prefix.to_string(), styles.footer_hint())),
            hint_line,
        ])
    } else {
        Text::from(hint_line)
    };

    f.render_widget(Paragraph::new(text).block(block), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &'static str, label: &'static str) -> FooterHintEntry {
        FooterHintEntry { key, label }
    }

    #[test]
    fn wide_width_fits_all_entries_without_ellipsis() {
        let entries = &[
            entry("t", "Switch"),
            entry("f", "Filter"),
            entry("d", "Downloads"),
        ];
        let (fitted, truncated) = fit_footer_entries(entries, 200);
        assert_eq!(fitted.len(), 3);
        assert!(!truncated);
        assert!(line_width(&fitted, false) <= 200);
    }

    #[test]
    fn narrow_width_drops_low_priority_entries() {
        let entries = &[
            entry("t", "Switch"),
            entry("f", "Filter"),
            entry("Ctrl+←/→", "Resize"),
        ];
        let (fitted, truncated) = fit_footer_entries(entries, 40);
        assert!(truncated);
        assert!(!fitted.is_empty());
        assert!(line_width(&fitted, true) <= 40);
        assert!(fitted[0].key == "t");
    }

    #[test]
    fn very_narrow_width_keeps_help_anchor() {
        let entries = &[entry("t", "Switch"), entry("f", "Filter")];
        let (fitted, truncated) = fit_footer_entries(entries, 12);
        assert!(!truncated);
        assert!(fitted.is_empty());
        assert!(line_width(&[], false) <= 12);
    }

    #[test]
    fn wide_glyphs_use_display_width_not_byte_length() {
        let narrow = entry("↑↓", "Select");
        let ascii = entry("ab", "cd");
        assert!(entry_display_width(&narrow) > entry_display_width(&ascii));
        let (fitted, _) = fit_footer_entries(&[narrow, ascii], 20);
        assert!(fitted.len() <= 2);
    }

    #[test]
    fn footer_hint_line_includes_anchor_text() {
        let theme = crate::tui::theme::resolve_theme_or_default("terminal");
        let styles = RommStyles::new(theme.as_ref());
        let line = footer_hint_line(&[entry("t", "Switch")], 200, &styles);
        let text = line.to_string();
        assert!(text.contains("? / F1"));
        assert!(text.contains("Help"));
    }

    #[test]
    fn truncated_line_includes_ellipsis() {
        let theme = crate::tui::theme::resolve_theme_or_default("terminal");
        let styles = RommStyles::new(theme.as_ref());
        let entries = &[
            entry("t", "Switch"),
            entry("f", "Filter"),
            entry("Ctrl+u", "Upload"),
            entry("/", "Search"),
            entry(",", "Settings"),
            entry("d", "Downloads"),
            entry("Ctrl+←/→", "Resize"),
        ];
        let line = footer_hint_line(entries, 50, &styles);
        assert!(line.to_string().contains('…'));
    }
}
