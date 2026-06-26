//! Static keyboard shortcut reference for the help overlay.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::RommStyles;

/// Minimum inner width (each column ~36 chars) for side-by-side layout.
const TWO_COL_MIN_INNER_WIDTH: u16 = 72;
/// Key column width for aligned shortcut tables.
const KEY_COL_WIDTH: usize = 20;

struct HelpEntry {
    key: &'static str,
    desc: &'static str,
}

struct HelpSection {
    title: &'static str,
    entries: &'static [HelpEntry],
}

const SECTIONS_LEFT: &[HelpSection] = &[
    HelpSection {
        title: "Global",
        entries: &[
            HelpEntry {
                key: "F1 / ?",
                desc: "This help (when not typing in a field)",
            },
            HelpEntry {
                key: "/",
                desc: "Server-wide ROM search overlay (toggle)",
            },
            HelpEntry {
                key: "d",
                desc: "Downloads overlay (toggle)",
            },
            HelpEntry {
                key: ",",
                desc: "Settings overlay (toggle)",
            },
            HelpEntry {
                key: "Ctrl+r",
                desc: "Rescan library on server (waits; refreshes games)",
            },
            HelpEntry {
                key: "q / Ctrl+c",
                desc: "Quit",
            },
        ],
    },
    HelpSection {
        title: "Library",
        entries: &[
            HelpEntry {
                key: "↑ / k, ↓ / j",
                desc: "Move in list or game rows",
            },
            HelpEntry {
                key: "← / h",
                desc: "Back to console list (games view)",
            },
            HelpEntry {
                key: "→ / l, Tab",
                desc: "Switch panel or view",
            },
            HelpEntry {
                key: "Enter",
                desc: "Open games list or game detail",
            },
            HelpEntry {
                key: "f",
                desc: "Filter focused pane (consoles, collections, games)",
            },
            HelpEntry {
                key: "t",
                desc: "Switch consoles / collections",
            },
            HelpEntry {
                key: "Ctrl+u",
                desc: "Upload ROM (consoles; Ctrl+s: rescan after)",
            },
            HelpEntry {
                key: "Ctrl+←/→",
                desc: "Resize console/games split",
            },
            HelpEntry {
                key: "Esc",
                desc: "Quit from list; back from games",
            },
        ],
    },
    HelpSection {
        title: "Search overlay",
        entries: &[
            HelpEntry {
                key: "Arrows, typing",
                desc: "Edit query and move in results",
            },
            HelpEntry {
                key: "Enter",
                desc: "Run search; open game if query matches last search",
            },
            HelpEntry {
                key: "Esc",
                desc: "Clear results or close overlay",
            },
            HelpEntry {
                key: "d / , / /",
                desc: "Typed into query while overlay is open",
            },
        ],
    },
];

const SECTIONS_RIGHT: &[HelpSection] = &[
    HelpSection {
        title: "Game detail",
        entries: &[
            HelpEntry {
                key: "Enter",
                desc: "Download",
            },
            HelpEntry {
                key: "o",
                desc: "Open cover image",
            },
            HelpEntry {
                key: "m",
                desc: "Match metadata (edit search title first)",
            },
            HelpEntry {
                key: "t",
                desc: "Toggle technical details",
            },
            HelpEntry {
                key: "Shift+U",
                desc: "Unmatch metadata",
            },
            HelpEntry {
                key: "Ctrl+←/→",
                desc: "Resize cover panel",
            },
            HelpEntry {
                key: "Esc",
                desc: "Back",
            },
            HelpEntry {
                key: "q",
                desc: "Quit",
            },
        ],
    },
    HelpSection {
        title: "Downloads overlay",
        entries: &[HelpEntry {
            key: "Esc / d",
            desc: "Close",
        }],
    },
    HelpSection {
        title: "Settings overlay",
        entries: &[
            HelpEntry {
                key: "Tab / Shift+Tab",
                desc: "Switch tab",
            },
            HelpEntry {
                key: "← / h, → / l",
                desc: "Switch tab",
            },
            HelpEntry {
                key: "↑ / k, ↓ / j",
                desc: "Move",
            },
            HelpEntry {
                key: "Enter",
                desc: "Edit/toggle setting, open pickers, auth wizard",
            },
            HelpEntry {
                key: "s",
                desc: "Save config to disk",
            },
            HelpEntry {
                key: "Esc",
                desc: "Close overlay (prompts if unsaved)",
            },
            HelpEntry {
                key: ",",
                desc: "Close overlay",
            },
            HelpEntry {
                key: "q",
                desc: "Quit",
            },
        ],
    },
    HelpSection {
        title: "Setup wizard",
        entries: &[HelpEntry {
            key: "(on screen)",
            desc: "Follow prompts; Esc returns when offered",
        }],
    },
];

/// Result of a key press while the help overlay is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardHelpInput {
    Close,
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageUpLarge,
    ScrollPageDown,
    ScrollPageDownLarge,
    Ignore,
}

pub fn map_keyboard_help_key(code: crossterm::event::KeyCode) -> KeyboardHelpInput {
    use crossterm::event::KeyCode;
    match code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::F(1) | KeyCode::Char('?') => {
            KeyboardHelpInput::Close
        }
        KeyCode::Up | KeyCode::Char('k') => KeyboardHelpInput::ScrollUp,
        KeyCode::Down | KeyCode::Char('j') => KeyboardHelpInput::ScrollDown,
        KeyCode::PageUp => KeyboardHelpInput::ScrollPageUp,
        KeyCode::PageDown => KeyboardHelpInput::ScrollPageDown,
        KeyCode::Home => KeyboardHelpInput::ScrollPageUpLarge,
        KeyCode::End => KeyboardHelpInput::ScrollPageDownLarge,
        _ => KeyboardHelpInput::Ignore,
    }
}

fn section_lines(sections: &[HelpSection], styles: &RommStyles<'_>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (i, section) in sections.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            section.title.to_string(),
            styles.label().add_modifier(Modifier::BOLD),
        )));
        for entry in section.entries {
            lines.push(entry_line(entry, styles));
        }
    }
    lines
}

fn entry_line(entry: &HelpEntry, styles: &RommStyles<'_>) -> Line<'static> {
    let key = if entry.key.len() > KEY_COL_WIDTH {
        format!("  {}", entry.key)
    } else {
        format!("  {:<width$}", entry.key, width = KEY_COL_WIDTH)
    };
    Line::from(vec![
        Span::styled(key, styles.label()),
        Span::styled(entry.desc.to_string(), styles.text()),
    ])
}

fn footer_lines(styles: &RommStyles<'_>, scrollable: bool) -> Vec<Line<'static>> {
    let hint = if scrollable {
        "Esc / Enter / F1 / ? close   ↑↓ / PgUp/PgDn scroll"
    } else {
        "Esc / Enter / F1 / ? to close"
    };
    vec![Line::from(Span::styled(
        hint.to_string(),
        styles.footer_hint(),
    ))]
}

fn popup_rect(area: Rect) -> Rect {
    let popup_w = (area.width * 9 / 10).max(48).min(area.width);
    let popup_h = (area.height * 9 / 10).max(12).min(area.height);
    Rect {
        x: area.width.saturating_sub(popup_w) / 2,
        y: area.height.saturating_sub(popup_h) / 2,
        width: popup_w,
        height: popup_h,
    }
}

fn use_two_column_layout(inner_width: u16, left_h: usize, right_h: usize, visible_h: u16) -> bool {
    inner_width >= TWO_COL_MIN_INNER_WIDTH
        && left_h <= visible_h as usize
        && right_h <= visible_h as usize
}

fn render_scrolled_column(
    f: &mut Frame,
    area: Rect,
    lines: &[Line<'static>],
    scroll: u16,
    styles: &RommStyles<'_>,
) {
    let visible_h = area.height as usize;
    let max_scroll = lines.len().saturating_sub(visible_h);
    let scroll = scroll.min(max_scroll as u16) as usize;
    let visible: Vec<Line<'static>> = lines.iter().skip(scroll).take(visible_h).cloned().collect();
    f.render_widget(
        Paragraph::new(Text::from(visible)).style(styles.text()),
        area,
    );
}

pub fn render_keyboard_help(f: &mut Frame, area: Rect, styles: &RommStyles, scroll: u16) -> u16 {
    let popup_area = popup_rect(area);
    styles.fill_surface(f, popup_area);

    let block = styles.panel_block("Keyboard shortcuts");
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let left_lines = section_lines(SECTIONS_LEFT, styles);
    let right_lines = section_lines(SECTIONS_RIGHT, styles);
    let footer = footer_lines(styles, false);
    let visible_h = inner.height;

    if use_two_column_layout(inner.width, left_lines.len(), right_lines.len(), visible_h) {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(1)
            .split(inner);

        f.render_widget(
            Paragraph::new(Text::from(left_lines)).style(styles.text()),
            columns[0],
        );
        let mut right_with_footer = right_lines;
        right_with_footer.extend(footer);
        f.render_widget(
            Paragraph::new(Text::from(right_with_footer)).style(styles.text()),
            columns[1],
        );
        0
    } else {
        let mut body_lines = section_lines(SECTIONS_LEFT, styles);
        body_lines.extend(section_lines(SECTIONS_RIGHT, styles));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(2)])
            .margin(1)
            .split(inner);

        let body_h = chunks[0].height as usize;
        let scrollable = body_lines.len() > body_h;
        let max_scroll = body_lines.len().saturating_sub(body_h);
        let clamped = scroll.min(max_scroll as u16);

        render_scrolled_column(f, chunks[0], &body_lines, clamped, styles);
        f.render_widget(
            Paragraph::new(Text::from(footer_lines(styles, scrollable))).style(styles.text()),
            chunks[1],
        );
        clamped
    }
}

/// Apply scroll input; `page_lines` is the visible body height of the help popup.
pub fn apply_keyboard_help_scroll(scroll: u16, input: KeyboardHelpInput, page_lines: u16) -> u16 {
    let page = page_lines.max(1);
    match input {
        KeyboardHelpInput::ScrollUp => scroll.saturating_sub(1),
        KeyboardHelpInput::ScrollDown => scroll.saturating_add(1),
        KeyboardHelpInput::ScrollPageUp => scroll.saturating_sub(page),
        KeyboardHelpInput::ScrollPageDown => scroll.saturating_add(page),
        KeyboardHelpInput::ScrollPageUpLarge => 0,
        KeyboardHelpInput::ScrollPageDownLarge => u16::MAX,
        KeyboardHelpInput::Close | KeyboardHelpInput::Ignore => scroll,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    use crate::tui::theme::{resolve_theme_or_default, RommStyles};
    use romm_api::config::default_theme_id;

    use super::*;

    #[test]
    fn keyboard_help_clears_background_inside_popup() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let theme = resolve_theme_or_default(&default_theme_id());
        let styles = RommStyles::new(theme.as_ref());

        terminal
            .draw(|frame| {
                let area = frame.area();
                let leak_area = Rect {
                    x: 20,
                    y: 10,
                    width: 40,
                    height: 20,
                };
                frame.render_widget(Paragraph::new("Air Raid || Background leak"), leak_area);
                render_keyboard_help(frame, area, &styles, 0);
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let text = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(text.contains("Global"), "help text should render");
        assert!(text.contains("Game detail"), "right column should render");
    }

    #[test]
    fn narrow_terminal_uses_single_column() {
        let backend = TestBackend::new(60, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let theme = resolve_theme_or_default(&default_theme_id());
        let styles = RommStyles::new(theme.as_ref());

        terminal
            .draw(|frame| {
                render_keyboard_help(frame, frame.area(), &styles, 0);
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let text = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(
            text.contains("scroll"),
            "narrow layout should show scroll hint"
        );
    }

    #[test]
    fn apply_scroll_respects_page_size() {
        assert_eq!(
            apply_keyboard_help_scroll(10, KeyboardHelpInput::ScrollPageUp, 5),
            5
        );
        assert_eq!(
            apply_keyboard_help_scroll(3, KeyboardHelpInput::ScrollUp, 8),
            2
        );
    }
}
