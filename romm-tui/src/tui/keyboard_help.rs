//! Static keyboard shortcut reference for the help overlay.

use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::RommStyles;

pub const KEYBOARD_HELP_TEXT: &str = "\
Global
  F1 / ?           This help (when not typing in a field)
  /                Server-wide ROM search overlay (toggle)
  d                Downloads overlay (toggle; disabled while typing in a field)
  ,                Settings overlay (toggle; disabled while typing in a field)
  Ctrl+r           Rescan library on server (waits until done; refreshes games list)
  q / Ctrl+c       Quit

Library (consoles / games)
  Up / k, Down / j   Move in list or game rows
  Left / h           Back to console list (games view)
  Right / l, Tab     Switch panel or view
  Enter              Open games list or game detail
  t                  Switch consoles / collections
  Ctrl+u             Upload ROM (Consoles list; path browser; Ctrl+s: rescan after upload)
  Ctrl+Left/Right    Resize console/games split
  Esc                Quit from list view; back from games view

Search overlay
  Arrows, typing     Edit query and move in results
  Enter              Run search; open game only if the query matches the last search
  Esc                Clear results or close overlay
  d / , /            Typed into query (overlay shortcuts disabled while typing)";

const KEYBOARD_HELP_TEXT_RIGHT: &str = "\
Game detail
  Enter              Download
  o                  Open cover image
  m                  Toggle technical details
  Ctrl+Left/Right    Resize cover panel
  Esc                Back
  q                  Quit

Downloads overlay
  Esc / d            Close

Settings overlay
  Tab / Shift+Tab,
  Left / h, Right / l Switch tab
  Up / k, Down / j   Move
  Enter              Edit/toggle selected setting, open pickers, or open auth wizard
  s                  Save config to disk
  Esc                Close overlay (prompts if unsaved)
  ,                  Close overlay
  q                  Quit

Setup wizard
  Follow on-screen prompts; Esc returns when offered.

Press Esc, Enter, F1, or ? to close this help.";

pub fn render_keyboard_help(f: &mut Frame, area: Rect, styles: &RommStyles) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_w = (area.width * 9 / 10).max(72).min(area.width);
    let popup_h = (area.height * 9 / 10).max(20).min(area.height);
    let popup_area = Rect {
        x: area.width.saturating_sub(popup_w) / 2,
        y: area.height.saturating_sub(popup_h) / 2,
        width: popup_w,
        height: popup_h,
    };
    styles.fill_surface(f, popup_area);

    let block = styles.panel_block("Keyboard shortcuts");
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(inner);

    let wrap = Wrap { trim: true };
    f.render_widget(
        Paragraph::new(KEYBOARD_HELP_TEXT)
            .style(styles.text())
            .wrap(wrap),
        columns[0],
    );
    f.render_widget(
        Paragraph::new(KEYBOARD_HELP_TEXT_RIGHT)
            .style(styles.text())
            .wrap(wrap),
        columns[1],
    );
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
                render_keyboard_help(frame, area, &styles);
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let popup_w = 108_u16;
        let popup_h = 36_u16;
        let popup_x = 6_u16;
        let popup_y = 2_u16;
        let block = styles.panel_block("Keyboard shortcuts");
        let inner = block.inner(Rect {
            x: popup_x,
            y: popup_y,
            width: popup_w,
            height: popup_h,
        });

        for y in inner.y..inner.y.saturating_add(inner.height) {
            for x in inner.x..inner.x.saturating_add(inner.width) {
                let cell = buffer[(x, y)].symbol();
                assert_ne!(
                    cell, "|",
                    "background table content leaked into help popup at ({x},{y})"
                );
            }
        }

        let text = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(text.contains("Global"), "help text should render");
        assert!(text.contains("Game detail"), "right column should render");
    }
}
