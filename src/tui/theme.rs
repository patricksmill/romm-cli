//! Semantic TUI styling backed by ratatui-themekit presets.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
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

    /// Paint the full frame background when the theme defines one.
    pub fn fill_background(&self, f: &mut Frame, area: Rect) {
        if self.has_immersive_background() {
            f.render_widget(
                Paragraph::new("").style(self.background()),
                area,
            );
        }
    }

    /// Fill a region (e.g. popup) with the panel surface color.
    pub fn fill_surface(&self, f: &mut Frame, area: Rect) {
        f.render_widget(Paragraph::new("").style(self.surface_text()), area);
    }

    pub fn background(&self) -> Style {
        Style::default().bg(self.theme.background())
    }

    pub fn surface(&self) -> Style {
        Style::default().bg(self.theme.surface())
    }

    fn surface_text(&self) -> Style {
        self.surface().fg(self.theme.text())
    }

    pub fn text(&self) -> Style {
        Style::default().fg(self.theme.text())
    }

    pub fn stripe(&self) -> Style {
        Style::default()
            .fg(self.theme.text())
            .bg(self.theme.stripe())
    }

    pub fn border(&self) -> Style {
        Style::default().fg(self.theme.border())
    }

    pub fn border_accent(&self) -> Style {
        Style::default().fg(self.theme.accent())
    }

    pub fn selection(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .bg(self.theme.stripe())
            .add_modifier(Modifier::BOLD)
    }

    /// Style for a table/list row: selected, zebra odd, or default.
    pub fn row(&self, index: usize, selected: bool) -> Style {
        if selected {
            self.selection()
        } else if index % 2 == 1 {
            self.stripe()
        } else {
            self.text()
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
        Style::default().fg(self.theme.text_bright())
    }

    pub fn border_focus(&self) -> Style {
        self.border_accent()
    }

    pub fn footer_hint(&self) -> Style {
        Style::default().fg(self.theme.text_dim())
    }

    /// Bordered panel with themed surface fill.
    pub fn panel_block<'b>(&self, title: impl Into<Line<'b>>) -> Block<'b> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(self.border())
            .style(self.surface_text())
    }

    /// Bordered panel without a title.
    pub fn panel_block_untitled(&self) -> Block<'_> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(self.border())
            .style(self.surface_text())
    }

    /// Header strip with bottom border only.
    pub fn header_block(&self) -> Block<'_> {
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(self.border())
            .style(self.surface_text())
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
}
