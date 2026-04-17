//! Static keyboard shortcut reference for the help overlay.

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

pub const KEYBOARD_HELP_TEXT: &str = "\
Global
  F1          This help (anywhere)
  ?           This help (when not typing in a field)
  d           Downloads overlay (toggle; disabled while typing in a field)

Main menu
  Up / k, Down / j   Move
  Enter              Open selected item
  Esc / q            Quit

Library (consoles / games)
  Up / k, Down / j   Move in list or game rows
  Left / h           Back to console list (games view)
  Right / l, Tab     Switch panel or view
  /                  Filter games (games view)
  Enter              After typing a filter: browse matches; Esc clears filter
  f                  Jump to match (games view)
  Tab                Next jump match (jump mode)
  Enter              Open games list or game detail
  t                  Switch consoles / collections
  Esc                Back or main menu
  q                  Quit

Search
  Arrows, typing     Edit query and move in results
  Enter              Run search or open game
  Esc                Clear results or main menu

Game detail
  Enter              Download
  o                  Open cover image
  m                  Toggle technical details
  Esc                Back
  q                  Quit

Downloads overlay
  Esc / d            Close

Settings
  Up / k, Down / j   Move
  Enter              Edit row or open auth wizard
  s                  Save config to disk
  Esc                Main menu
  q                  Quit

API browse
  Up / k, Down / j   Move
  Left / h, Right / l, Tab   Switch panes
  Enter              Open endpoint or switch pane
  Esc                Main menu

Execute request
  Tab / Shift+Tab    Next / previous field
  Typing             Edit fields
  Enter              Send request
  Esc                Back to browse

JSON / table result
  Up / k, Down / j, PgUp / PgDn   Scroll
  t                  Toggle JSON / table (when applicable)
  Enter              Open selected row (table)
  Esc                Back to browse
  q                  Quit

Result detail
  Arrows, PgUp / PgDn   Scroll
  o                  Open image URL
  Esc                Back
  q                  Quit

Setup wizard
  Follow on-screen prompts; Esc returns when offered.

Press Esc, Enter, F1, or ? to close this help.
";

pub fn render_keyboard_help(f: &mut Frame, area: Rect) {
    let popup_w = (area.width * 4 / 5).max(40).min(area.width);
    let popup_h = (area.height * 4 / 5).max(15).min(area.height);
    let popup_area = Rect {
        x: area.width.saturating_sub(popup_w) / 2,
        y: area.height.saturating_sub(popup_h) / 2,
        width: popup_w,
        height: popup_h,
    };
    f.render_widget(Clear, popup_area);
    let block = Block::default()
        .title("Keyboard shortcuts")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(KEYBOARD_HELP_TEXT)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, popup_area);
}
