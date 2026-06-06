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
  /                  Close overlay

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

Press Esc, Enter, F1, or ? to close this help.
";

pub fn render_keyboard_help(f: &mut Frame, area: Rect, styles: &RommStyles) {
    let popup_w = (area.width * 4 / 5).max(40).min(area.width);
    let popup_h = (area.height * 4 / 5).max(15).min(area.height);
    let popup_area = Rect {
        x: area.width.saturating_sub(popup_w) / 2,
        y: area.height.saturating_sub(popup_h) / 2,
        width: popup_w,
        height: popup_h,
    };
    styles.fill_surface(f, popup_area);
    let block = styles.panel_block("Keyboard shortcuts");
    let paragraph = Paragraph::new(KEYBOARD_HELP_TEXT)
        .block(block)
        .style(styles.text())
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, popup_area);
}
