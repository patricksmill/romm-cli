//! Semantic TUI styling backed by ratatui-themekit presets.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;
use ratatui_themekit::{available_theme_ids, resolve_theme, Theme};

use crate::config::{default_theme_id, DEFAULT_THEME_ID};

/// Status message severity for themed TUI feedback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageTone {
    Success,
    Error,
    Warning,
    Info,
}

/// Resolve a theme ID, falling back to [`DEFAULT_THEME_ID`] for unknown values.
pub fn resolve_theme_or_default(id: &str) -> Box<dyn Theme> {
    if ratatui_themekit::no_color_active() {
        return resolve_theme(id);
    }
    let known = id == "no-color" || available_theme_ids().contains(&id);
    if !known {
        tracing::warn!(theme = id, "unknown theme ID, using {DEFAULT_THEME_ID}");
        return resolve_theme(DEFAULT_THEME_ID);
    }
    resolve_theme(id)
}

/// Human-readable name for a theme ID (uses fallback resolution).
pub fn theme_display_name(id: &str) -> String {
    resolve_theme_or_default(id).name().to_string()
}

/// Cycle to the next built-in theme ID (wraps).
pub fn next_theme_id(current: &str) -> String {
    let ids: Vec<&str> = available_theme_ids();
    if ids.is_empty() {
        return default_theme_id();
    }
    let idx = ids.iter().position(|id| *id == current).unwrap_or(0);
    ids[(idx + 1) % ids.len()].to_string()
}

/// Cycle to the previous built-in theme ID (wraps).
pub fn prev_theme_id(current: &str) -> String {
    let ids: Vec<&str> = available_theme_ids();
    if ids.is_empty() {
        return default_theme_id();
    }
    let idx = ids.iter().position(|id| *id == current).unwrap_or(0);
    let len = ids.len();
    ids[(idx + len - 1) % len].to_string()
}

/// App-level semantic styles mapped onto a ratatui-themekit theme.
pub struct RommStyles<'a> {
    theme: &'a dyn Theme,
}

impl<'a> RommStyles<'a> {
    pub fn new(theme: &'a dyn Theme) -> Self {
        Self { theme }
    }

    pub fn theme(&self) -> &dyn Theme {
        self.theme
    }

    /// Whether this theme defines a real background (vs terminal default).
    pub fn has_immersive_background(&self) -> bool {
        !matches!(self.theme.background(), Color::Reset)
    }

    /// Use the terminal emulator palette without forced panel/surface fills.
    pub fn uses_native_terminal(&self) -> bool {
        !self.has_immersive_background()
    }

    /// Paint the full frame background when the theme defines one.
    pub fn fill_background(&self, f: &mut Frame, area: Rect) {
        if self.has_immersive_background() {
            f.render_widget(Paragraph::new("").style(self.background()), area);
        }
    }

    /// Fill a region (e.g. popup) with the panel surface color.
    pub fn fill_surface(&self, f: &mut Frame, area: Rect) {
        if self.uses_native_terminal() {
            return;
        }
        f.render_widget(Paragraph::new("").style(self.surface_text()), area);
    }

    pub fn background(&self) -> Style {
        Style::default().bg(self.theme.background())
    }

    pub fn surface(&self) -> Style {
        if self.uses_native_terminal() {
            Style::default()
        } else {
            Style::default().bg(self.theme.surface())
        }
    }

    fn surface_text(&self) -> Style {
        if self.uses_native_terminal() {
            Style::default()
        } else {
            self.surface().fg(self.theme.text())
        }
    }

    pub fn text(&self) -> Style {
        if self.uses_native_terminal() {
            Style::default()
        } else {
            Style::default().fg(self.theme.text())
        }
    }

    pub fn stripe(&self) -> Style {
        if self.uses_native_terminal() {
            self.text()
        } else {
            Style::default()
                .fg(self.theme.text())
                .bg(self.theme.stripe())
        }
    }

    pub fn border(&self) -> Style {
        if self.uses_native_terminal() {
            Style::default().fg(self.theme.border())
        } else {
            // Borders sit on the canvas, not the panel surface, so edges stay visible.
            Style::default()
                .fg(self.theme.text_dim())
                .bg(self.theme.background())
        }
    }

    pub fn border_accent(&self) -> Style {
        let mut style = Style::default().fg(self.theme.accent());
        if self.has_immersive_background() {
            style = style.bg(self.theme.background());
        }
        style
    }

    pub fn selection(&self) -> Style {
        if self.uses_native_terminal() {
            Style::default()
                .fg(self.theme.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(self.theme.accent())
                .bg(self.theme.stripe())
                .add_modifier(Modifier::BOLD)
        }
    }

    /// Style for a table/list row: selected, zebra odd, or default.
    pub fn row(&self, index: usize, selected: bool) -> Style {
        if selected {
            self.selection()
        } else if self.uses_native_terminal() || index.is_multiple_of(2) {
            self.text()
        } else {
            self.stripe()
        }
    }

    pub fn label(&self) -> Style {
        Style::default().fg(self.theme.info())
    }

    pub fn success(&self) -> Style {
        Style::default().fg(self.theme.success())
    }

    pub fn error(&self) -> Style {
        Style::default().fg(self.theme.error())
    }

    pub fn warning(&self) -> Style {
        Style::default().fg(self.theme.warning())
    }

    pub fn muted(&self) -> Style {
        Style::default().fg(self.theme.text_dim())
    }

    pub fn primary_text(&self) -> Style {
        if self.uses_native_terminal() {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.text_bright())
        }
    }

    pub fn border_focus(&self) -> Style {
        self.border_accent()
    }

    pub fn footer_hint(&self) -> Style {
        Style::default().fg(self.theme.text_dim())
    }

    /// Bordered panel with themed surface fill.
    pub fn panel_block<'b>(&self, title: impl Into<Line<'b>>) -> Block<'b> {
        let border_type = if self.uses_native_terminal() {
            BorderType::Plain
        } else {
            BorderType::Rounded
        };
        let mut block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(self.border())
            .title_style(
                Style::default()
                    .fg(self.theme.accent())
                    .add_modifier(Modifier::BOLD),
            );
        if !self.uses_native_terminal() {
            block = block.style(self.surface());
        }
        block
    }

    /// Bordered panel without a title.
    pub fn panel_block_untitled(&self) -> Block<'_> {
        let border_type = if self.uses_native_terminal() {
            BorderType::Plain
        } else {
            BorderType::Rounded
        };
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(self.border());
        if !self.uses_native_terminal() {
            block = block.style(self.surface());
        }
        block
    }

    /// Header strip with bottom border only.
    pub fn header_block(&self) -> Block<'_> {
        let mut block = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain)
            .border_style(self.border());
        if !self.uses_native_terminal() {
            block = block.style(self.surface());
        }
        block
    }

    pub fn color_success(&self) -> Color {
        self.theme.success()
    }

    pub fn color_error(&self) -> Color {
        self.theme.error()
    }

    pub fn color_warning(&self) -> Color {
        self.theme.warning()
    }

    pub fn color_info(&self) -> Color {
        self.theme.info()
    }

    pub fn tone(&self, tone: MessageTone) -> Style {
        match tone {
            MessageTone::Success => self.success(),
            MessageTone::Error => self.error(),
            MessageTone::Warning => self.warning(),
            MessageTone::Info => self.label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_unknown_falls_back_to_terminal() {
        std::env::remove_var("NO_COLOR");
        let theme = resolve_theme_or_default("not-a-theme");
        assert_eq!(theme.id(), "terminal");
    }

    #[test]
    fn dracula_has_immersive_background_and_selection_contrast() {
        std::env::remove_var("NO_COLOR");
        let theme = resolve_theme_or_default("dracula");
        let styles = RommStyles::new(theme.as_ref());
        assert!(styles.has_immersive_background());
        assert_ne!(styles.selection().fg, None);
        assert_ne!(styles.selection().bg, None);
    }

    #[test]
    fn terminal_theme_respects_native_terminal_colors() {
        std::env::remove_var("NO_COLOR");
        let theme = resolve_theme_or_default("terminal");
        let styles = RommStyles::new(theme.as_ref());
        assert!(styles.uses_native_terminal());
        assert_eq!(styles.surface().bg, None);
        assert_eq!(styles.selection().bg, None);
        assert_eq!(styles.text().fg, None);
    }

    #[test]
    fn dracula_border_contrasts_with_surface() {
        std::env::remove_var("NO_COLOR");
        let theme = resolve_theme_or_default("dracula");
        let styles = RommStyles::new(theme.as_ref());
        let border = styles.border();
        assert_eq!(border.bg, Some(theme.background()));
        assert_ne!(border.fg, Some(theme.surface()));
    }
}
